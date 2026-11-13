//! Email addresses as a type that can only hold valid, normalised values, and the mail that
//! sign-in links travel in.
//!
//! The parser is the only way to make an [`EmailAddress`]. Mail senders and handlers receive
//! an already-checked address, so no code builds an address string by hand. Templates are fixed
//! strings with escaped values; the only variable parts are the link and the recipient.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use thiserror::Error;

use crate::auth::{clock::Clock, config::AppEnv, limits::AuthLimiter, tokens::Purpose};

/// The longest address the parser accepts, in bytes.
pub const MAX_EMAIL_LEN: usize = 254;

/// The address is not one we accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("the email address is not valid")]
pub struct EmailError;

/// A valid, lower-cased email address. It is never printed in full by `Debug`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Checks and normalises an address: trims, lower-cases, and enforces the rules of R4.3.
    ///
    /// Dots and `+tags` are kept, so two different mailboxes stay different (R4.4).
    ///
    /// # Errors
    ///
    /// Returns [`EmailError`] when the input breaks any rule.
    pub fn parse(input: &str) -> Result<Self, EmailError> {
        let normalised = input.trim().to_ascii_lowercase();
        if normalised.len() > MAX_EMAIL_LEN || !input.trim().is_ascii() {
            return Err(EmailError);
        }
        let forbidden = normalised.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | ',' | ';' | '"')
        });
        if forbidden {
            return Err(EmailError);
        }
        let (local, domain) = normalised.split_once('@').ok_or(EmailError)?;
        let one_at_only = !domain.contains('@');
        let domain_ok = domain.contains('.') && domain.split('.').all(|label| !label.is_empty());
        if local.is_empty() || !one_at_only || !domain_ok {
            return Err(EmailError);
        }
        Ok(Self(normalised))
    }

    /// The normalised address text. Use it only where the address is needed (for example in a
    /// database row or a mail `to` field), never in a log line.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for EmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EmailAddress([redacted])")
    }
}

/// Why a message was not sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MailError {
    /// The recipient is the demo account. The demo account never receives mail.
    #[error("mail to this address is suppressed")]
    Suppressed,
    /// The global daily cap is reached, or the counter store is down. Mail fails closed.
    #[error("mail is unavailable right now")]
    Unavailable,
    /// The log transport was requested in production.
    #[error("this mail transport is not allowed in this environment")]
    NotAllowed,
}

/// One message ready to send. The body holds a sign-in link, so `Debug` prints neither the body
/// nor the recipient.
#[derive(Clone)]
pub struct OutgoingMail {
    /// The one recipient.
    pub to: EmailAddress,
    /// The subject line.
    pub subject: String,
    /// The plain-text body.
    pub text: String,
    /// The HTML body, with every variable value escaped.
    pub html: String,
    /// Sent as the provider's idempotency key, so a retry cannot send twice.
    pub idempotency_key: String,
}

impl fmt::Debug for OutgoingMail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OutgoingMail")
            .field("to", &self.to)
            .field("subject", &self.subject)
            .finish_non_exhaustive()
    }
}

/// Something that delivers mail. Handlers depend on this trait, so tests can record messages
/// instead of sending them.
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Sends one message.
    ///
    /// # Errors
    ///
    /// Returns [`MailError`] when the message was not sent.
    async fn send(&self, msg: OutgoingMail) -> Result<(), MailError>;
}

/// Keeps every message in memory. Tests read them back to find the sign-in link.
#[derive(Default)]
pub struct MemorySender {
    sent: Mutex<Vec<OutgoingMail>>,
}

impl MemorySender {
    /// The messages sent so far, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<OutgoingMail> {
        self.sent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl EmailSender for MemorySender {
    async fn send(&self, msg: OutgoingMail) -> Result<(), MailError> {
        self.sent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(msg);
        Ok(())
    }
}

/// Writes the message to the development log, so a developer can click the link without
/// a mail provider. It exists only in development.
pub struct LogSender;

impl LogSender {
    /// Builds the log sender for the given environment.
    ///
    /// # Errors
    ///
    /// Returns [`MailError::NotAllowed`] in production, where mail must go through Resend.
    pub fn new(env: AppEnv) -> Result<Self, MailError> {
        match env {
            AppEnv::Development => Ok(Self),
            AppEnv::Production => Err(MailError::NotAllowed),
        }
    }
}

#[async_trait]
impl EmailSender for LogSender {
    async fn send(&self, msg: OutgoingMail) -> Result<(), MailError> {
        // Development only. The body holds the sign-in link on purpose, so it can be clicked.
        tracing::info!(
            target: "counterspell::dev_mail",
            subject = %msg.subject,
            body = %msg.text,
            "development mail (not sent)"
        );
        Ok(())
    }
}

/// Wraps another sender and applies the rules that must hold for every message: the demo
/// address never receives mail, and the global daily cap stops sending once it is reached.
pub struct GuardedSender {
    inner: Arc<dyn EmailSender>,
    limiter: AuthLimiter,
    daily_cap: u32,
    demo: Option<EmailAddress>,
    clock: Arc<dyn Clock>,
}

impl GuardedSender {
    /// Builds the guard. `demo` is the demo account's address, or `None` when the demo is off.
    #[must_use]
    pub fn new(
        inner: Arc<dyn EmailSender>,
        limiter: AuthLimiter,
        daily_cap: u32,
        demo: Option<EmailAddress>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            inner,
            limiter,
            daily_cap,
            demo,
            clock,
        }
    }
}

#[async_trait]
impl EmailSender for GuardedSender {
    async fn send(&self, msg: OutgoingMail) -> Result<(), MailError> {
        if self.demo.as_ref() == Some(&msg.to) {
            return Err(MailError::Suppressed);
        }
        let today = self.clock.now().date();
        self.limiter
            .hit_daily_mail_cap(today, self.daily_cap)
            .await
            .map_err(|_| MailError::Unavailable)?;
        self.inner.send(msg).await
    }
}

/// The sign-in link for a raw token. The token sits in the fragment, so it is not sent to the
/// server in the page request and not written to access logs.
#[must_use]
pub fn magic_link_url(public_base_url: &str, raw_token: &str) -> String {
    format!(
        "{}/auth/verify#token={raw_token}",
        public_base_url.trim_end_matches('/')
    )
}

/// Builds the message for one purpose. The link goes into both bodies, escaped in HTML.
#[must_use]
pub fn compose(
    purpose: Purpose,
    to: EmailAddress,
    link: &str,
    idempotency_key: &str,
) -> OutgoingMail {
    let (subject, intro) = match purpose {
        Purpose::MagicLink => (
            "Your Counterspell sign-in link",
            "Use this link to sign in to Counterspell.",
        ),
        Purpose::Verify => (
            "Confirm your Counterspell account",
            "Use this link to confirm your new Counterspell account.",
        ),
        Purpose::Reset => (
            "Reset your Counterspell password",
            "Use this link to choose a new Counterspell password.",
        ),
        Purpose::LinkEmail => (
            "Confirm this email for your Counterspell account",
            "Use this link to add this address to your signed-in Counterspell account.",
        ),
    };
    let note = "The link works once and expires in 15 minutes. If you did not ask for this, \
                ignore this email.";
    let html = format!(
        "<p>{}</p>\n<p><a href=\"{href}\">{href}</a></p>\n<p>{}</p>\n",
        escape_html(intro),
        escape_html(note),
        href = escape_html(link),
    );
    OutgoingMail {
        to,
        subject: subject.to_owned(),
        text: format!("{intro}\n\n{link}\n\n{note}\n"),
        html,
        idempotency_key: idempotency_key.to_owned(),
    }
}

/// Escapes the five characters that matter inside HTML text and attribute values.
fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn accepted(input: &str) -> String {
        EmailAddress::parse(input)
            .unwrap_or_else(|_| panic!("expected {input:?} to be accepted"))
            .as_str()
            .to_owned()
    }

    fn refused(input: &str) {
        assert_eq!(EmailAddress::parse(input), Err(EmailError), "{input:?}");
    }

    #[test]
    fn a_plain_address_is_trimmed_and_lower_cased() {
        assert_eq!(accepted("  Thandi@Example.CO  "), "thandi@example.co");
        assert_eq!(accepted("A@B.CO"), "a@b.co");
    }

    #[test]
    fn plus_tags_and_dots_are_kept_so_mailboxes_stay_distinct() {
        assert_eq!(accepted("a+x@b.co"), "a+x@b.co");
        assert_ne!(accepted("a.b@x.co"), accepted("ab@x.co"));
    }

    #[test]
    fn the_hostile_corpus_is_refused() {
        for input in [
            "a@b",
            "a@@b.co",
            "a b@c.co",
            "a@b.co\r\nBcc: x@y.z",
            "\"a\"@b.co",
            "a@b.co,c@d.co",
            "a<b>@c.co",
            "a@b;c.co",
            "a@.co",
            "a@b..co",
            "@b.co",
            "a@",
            "ünï@b.co",
            "",
            "   ",
        ] {
            refused(input);
        }
    }

    #[test]
    fn the_length_limit_is_254_bytes() {
        let at_limit = format!("{}@b.co", "a".repeat(249));
        assert_eq!(accepted(&at_limit).len(), 254);
        refused(&format!("{}@b.co", "a".repeat(250)));
    }

    proptest! {
        #[test]
        fn parsing_is_idempotent(input in "\\PC{0,300}") {
            if let Ok(first) = EmailAddress::parse(&input) {
                let second = EmailAddress::parse(first.as_str());
                prop_assert_eq!(second, Ok(first));
            }
        }

        #[test]
        fn accepted_values_follow_every_rule(input in "[ -~]{0,80}") {
            if let Ok(email) = EmailAddress::parse(&input) {
                let text = email.as_str();
                prop_assert!(text.len() <= MAX_EMAIL_LEN);
                prop_assert!(text.is_ascii());
                prop_assert_eq!(text.matches('@').count(), 1);
                prop_assert!(!text.chars().any(|c| c.is_whitespace() || c.is_control()));
                prop_assert!(!text.contains(['<', '>', ',', ';', '"']));
                prop_assert_eq!(text, text.to_ascii_lowercase());
                let (local, domain) = text.split_once('@').expect("one @");
                prop_assert!(!local.is_empty());
                prop_assert!(domain.contains('.'));
                prop_assert!(domain.split('.').all(|label| !label.is_empty()));
            }
        }
    }
}
