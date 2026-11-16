//! The OpenAI-compatible chat client. It is the only code that talks HTTP to a model provider.
//!
//! [`complete`] applies the per-attempt timeout around one transport call and logs only the
//! provider name, the model, the status code and the outcome. It never logs the key, the prompt,
//! or the provider's reply body.

use std::{future::Future, time::Duration};

use serde_json::{json, Value};
use thiserror::Error;

use crate::hints::{config::ProviderSpec, guard::Prompt};

/// Token counts a provider reports for one call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens in the reply.
    pub completion_tokens: u32,
}

/// A successful reply from a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResponse {
    /// The reply text, as the provider wrote it. The guard checks it before use.
    pub text: String,
    /// The token counts, or `None` when the provider did not report them.
    pub usage: Option<Usage>,
}

/// Why a provider call failed. The variants hold no provider text, so nothing sensitive is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProviderError {
    /// The provider did not answer within the timeout.
    #[error("the provider did not answer in time")]
    Timeout,
    /// The provider could not be reached.
    #[error("the provider could not be reached")]
    Transport,
    /// The provider answered with an error status.
    #[error("the provider answered with status {0}")]
    Status(u16),
    /// The provider's answer was not the expected JSON.
    #[error("the provider's answer could not be read")]
    BadBody,
}

/// One chat request, borrowed from the caller. The key is only read here, never stored.
#[derive(Debug, Clone, Copy)]
pub struct ChatRequest<'a> {
    /// The OpenAI-compatible base URL.
    pub base_url: &'a str,
    /// The bearer key.
    pub api_key: &'a str,
    /// The model name.
    pub model: &'a str,
    /// The system rules.
    pub system: &'a str,
    /// The user message.
    pub user: &'a str,
    /// The cap on reply tokens.
    pub max_tokens: u32,
}

/// Sends one chat request. The real implementation is [`ReqwestTransport`]; tests use a scripted one.
pub trait ChatTransport: Send + Sync {
    /// Sends `request` and returns the parsed reply.
    fn chat(
        &self,
        request: ChatRequest<'_>,
    ) -> impl Future<Output = Result<ChatResponse, ProviderError>> + Send;
}

/// The HTTP transport. It holds one shared client, so connections are reused.
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    /// Builds the shared HTTP client. Each call sets its own timeout in [`complete`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatTransport for ReqwestTransport {
    fn chat(
        &self,
        request: ChatRequest<'_>,
    ) -> impl Future<Output = Result<ChatResponse, ProviderError>> + Send {
        let url = format!("{}/chat/completions", request.base_url);
        let body = json!({
            "model": request.model,
            "messages": [
                { "role": "system", "content": request.system },
                { "role": "user", "content": request.user },
            ],
            "max_tokens": request.max_tokens,
            "temperature": 0.2,
        });
        let send = self
            .client
            .post(url)
            .bearer_auth(request.api_key)
            .json(&body)
            .send();
        async move {
            let response = send.await.map_err(|_| ProviderError::Transport)?;
            let status = response.status();
            if !status.is_success() {
                return Err(ProviderError::Status(status.as_u16()));
            }
            let value: Value = response.json().await.map_err(|_| ProviderError::BadBody)?;
            parse_chat_body(&value)
        }
    }
}

/// Reads the reply text and the optional token counts from an OpenAI-compatible body.
///
/// # Errors
///
/// Returns [`ProviderError::BadBody`] when `choices[0].message.content` is missing or not text.
pub fn parse_chat_body(body: &Value) -> Result<ChatResponse, ProviderError> {
    let text = body
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or(ProviderError::BadBody)?;
    let usage = body.get("usage").and_then(|usage| {
        Some(Usage {
            prompt_tokens: u32::try_from(usage.get("prompt_tokens")?.as_u64()?).ok()?,
            completion_tokens: u32::try_from(usage.get("completion_tokens")?.as_u64()?).ok()?,
        })
    });
    Ok(ChatResponse {
        text: text.to_owned(),
        usage,
    })
}

/// Calls one configured provider once, with `timeout` applied to the whole call, body included.
///
/// # Errors
///
/// Returns the [`ProviderError`] describing why the call failed.
pub async fn complete<T: ChatTransport>(
    transport: &T,
    spec: &ProviderSpec,
    prompt: &Prompt,
    max_tokens: u32,
    timeout: Duration,
) -> Result<ChatResponse, ProviderError> {
    let request = ChatRequest {
        base_url: &spec.base_url,
        api_key: spec.api_key.expose(),
        model: &spec.model,
        system: &prompt.system,
        user: &prompt.user,
        max_tokens,
    };
    let result = match tokio::time::timeout(timeout, transport.chat(request)).await {
        Ok(result) => result,
        Err(_elapsed) => Err(ProviderError::Timeout),
    };
    match &result {
        Ok(_) => {
            tracing::info!(provider = spec.name.as_str(), model = %spec.model, "hint provider answered")
        }
        Err(error) => {
            tracing::warn!(provider = spec.name.as_str(), model = %spec.model, outcome = %error, "hint provider call failed")
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hints::config::{Price, Redacted};
    use crate::hints::config::{ProviderName, ProviderSpec};
    use crate::hints::guard::{build_prompt, PromptContext, PromptKind};
    use crate::testkit::{ScriptedTransport, Step};
    use serde_json::json;
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tracing_subscriber::fmt::MakeWriter;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const SECRET_KEY: &str = "sk-secret-key-999";
    const PROMPT_MARKER: &str = "PROMPT-MARKER-4242";

    fn spec(base_url: &str) -> ProviderSpec {
        ProviderSpec {
            name: ProviderName::OpenAi,
            model: "gpt-6-luna".to_owned(),
            base_url: base_url.to_owned(),
            api_key: Redacted::new(SECRET_KEY.to_owned()),
            price: Price {
                input_per_million_micros: 100_000,
                output_per_million_micros: 500_000,
            },
        }
    }

    fn prompt() -> Prompt {
        let context = PromptContext {
            statement: "Count pairs.",
            code: PROMPT_MARKER,
            ghost: None,
            verdict_summary: "1 of 3 hidden tests failed",
        };
        build_prompt(PromptKind::Ghost { rung: 2 }, &context).expect("small code builds")
    }

    fn reply(text: &str) -> serde_json::Value {
        json!({
            "choices": [{ "message": { "role": "assistant", "content": text } }],
            "usage": { "prompt_tokens": 120, "completion_tokens": 30 }
        })
    }

    #[test]
    fn a_body_with_usage_gives_the_text_and_token_counts() {
        let response = parse_chat_body(&reply("Look at the second loop.")).expect("valid body");

        assert_eq!(response.text, "Look at the second loop.");
        assert_eq!(
            response.usage,
            Some(Usage {
                prompt_tokens: 120,
                completion_tokens: 30
            })
        );
    }

    #[test]
    fn a_body_without_usage_is_accepted_and_has_no_counts() {
        let body = json!({ "choices": [{ "message": { "content": "Try a map." } }] });

        let response = parse_chat_body(&body).expect("usage is optional");

        assert_eq!(response.text, "Try a map.");
        assert_eq!(response.usage, None);
    }

    #[test]
    fn a_body_without_a_message_is_a_bad_body() {
        let body = json!({ "choices": [] });

        assert_eq!(parse_chat_body(&body), Err(ProviderError::BadBody));
    }

    #[tokio::test]
    async fn a_stalled_provider_times_out() {
        let transport = ScriptedTransport::new(vec![Step::Stall]);

        let result = complete(
            &transport,
            &spec("https://example.invalid/v1"),
            &prompt(),
            300,
            Duration::from_millis(10),
        )
        .await;

        assert_eq!(result, Err(ProviderError::Timeout));
    }

    #[tokio::test]
    async fn a_successful_http_call_returns_the_reply() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer sk-secret-key-999"))
            .respond_with(ResponseTemplate::new(200).set_body_json(reply("A nudge.")))
            .expect(1)
            .mount(&server)
            .await;

        let response = complete(
            &ReqwestTransport::new(),
            &spec(&server.uri()),
            &prompt(),
            300,
            Duration::from_secs(5),
        )
        .await
        .expect("a 200 with a valid body succeeds");

        assert_eq!(response.text, "A nudge.");
    }

    #[tokio::test]
    async fn a_server_error_becomes_a_status_error_with_only_the_code() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal detail"))
            .mount(&server)
            .await;

        let result = complete(
            &ReqwestTransport::new(),
            &spec(&server.uri()),
            &prompt(),
            300,
            Duration::from_secs(5),
        )
        .await;

        assert_eq!(result, Err(ProviderError::Status(500)));
    }

    #[tokio::test]
    async fn an_unauthorised_call_becomes_a_status_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let result = complete(
            &ReqwestTransport::new(),
            &spec(&server.uri()),
            &prompt(),
            300,
            Duration::from_secs(5),
        )
        .await;

        assert_eq!(result, Err(ProviderError::Status(401)));
    }

    #[tokio::test]
    async fn a_body_that_is_not_json_is_a_bad_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>oops"))
            .mount(&server)
            .await;

        let result = complete(
            &ReqwestTransport::new(),
            &spec(&server.uri()),
            &prompt(),
            300,
            Duration::from_secs(5),
        )
        .await;

        assert_eq!(result, Err(ProviderError::BadBody));
    }

    /// A log writer that keeps everything in memory, so a test can read what was logged.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Ok(mut buffer) = self.0.lock() {
                buffer.extend_from_slice(bytes);
            }
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Capture {
        type Writer = Capture;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[tokio::test]
    async fn a_failed_call_logs_no_key_and_no_prompt_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string(PROMPT_MARKER))
            .mount(&server)
            .await;
        let capture = Capture::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(capture.clone())
            .with_ansi(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let _ = complete(
            &ReqwestTransport::new(),
            &spec(&server.uri()),
            &prompt(),
            300,
            Duration::from_secs(5),
        )
        .await;

        let logged = String::from_utf8_lossy(&capture.0.lock().expect("log lock")).into_owned();
        assert!(
            logged.contains("500"),
            "the status code is logged: {logged}"
        );
        assert!(!logged.contains(SECRET_KEY));
        assert!(!logged.contains(PROMPT_MARKER));
    }
}
