//! Session cookies: their names, their attributes, reading them, and sending `Set-Cookie` headers.
//!
//! On HTTPS the cookie name has the `__Host-` prefix, which forces `Secure`, `Path=/` and no
//! `Domain`. That stops a sibling subdomain from planting a cookie. Development over plain HTTP
//! uses the plain name and leaves out `Secure`.

use std::sync::{Arc, Mutex, PoisonError};

use axum::{
    extract::Request,
    http::{header, HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};

use super::{errors::AuthError, session::SESSION_IDLE};

/// The cookie names and attributes for this deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CookieSettings {
    secure: bool,
}

impl CookieSettings {
    /// Builds the settings. `secure` means the site is served over HTTPS.
    #[must_use]
    pub const fn new(secure: bool) -> Self {
        Self { secure }
    }

    /// Chooses the settings from the public base URL. An `https://` URL means secure cookies.
    #[must_use]
    pub fn for_base_url(base_url: &str) -> Self {
        Self::new(base_url.starts_with("https://"))
    }

    /// The name of the session cookie.
    #[must_use]
    pub fn session_name(self) -> &'static str {
        if self.secure {
            "__Host-cs_session"
        } else {
            "cs_session"
        }
    }

    /// The `Set-Cookie` value that stores `raw` for the full session lifetime.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the value cannot be a header (it always can for a
    /// token this server generated).
    pub fn session_value(self, raw: &str) -> Result<HeaderValue, AuthError> {
        let max_age = SESSION_IDLE.whole_seconds();
        header_value(&format!(
            "{}={raw}; {}; Max-Age={max_age}",
            self.session_name(),
            self.attributes()
        ))
    }

    /// The `Set-Cookie` value that removes the session cookie from the browser.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the header cannot be built.
    pub fn clear_session(self) -> Result<HeaderValue, AuthError> {
        header_value(&format!(
            "{}=; {}; Max-Age=0",
            self.session_name(),
            self.attributes()
        ))
    }

    fn attributes(self) -> &'static str {
        if self.secure {
            "Path=/; HttpOnly; Secure; SameSite=Lax"
        } else {
            "Path=/; HttpOnly; SameSite=Lax"
        }
    }
}

fn header_value(text: &str) -> Result<HeaderValue, AuthError> {
    HeaderValue::from_str(text).map_err(|_| AuthError::Internal)
}

/// Reads one cookie by name from the `Cookie` headers. Returns the first match.
#[must_use]
pub fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|line| line.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

/// A place for one `Set-Cookie` header. Extractors fill it, and the middleware adds it to the
/// response, so a renewed or cleared cookie reaches the browser even when a handler fails.
#[derive(Debug, Clone, Default)]
pub struct CookieSlot(Arc<Mutex<Option<HeaderValue>>>);

impl CookieSlot {
    /// Stores the header to send. A later call replaces an earlier one.
    pub fn set(&self, value: HeaderValue) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(value);
    }

    fn take(&self) -> Option<HeaderValue> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// Puts a [`CookieSlot`] into each request and adds whatever it holds to the response.
pub async fn session_cookie(mut request: Request, next: Next) -> Response {
    let slot = CookieSlot::default();
    request.extensions_mut().insert(slot.clone());
    let mut response = next.run(request).await;
    if let Some(value) = slot.take() {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_uses_the_host_prefix_and_secure_attributes() {
        let settings = CookieSettings::for_base_url("https://counterspell.example");
        assert_eq!(settings.session_name(), "__Host-cs_session");
        let value = settings.session_value("abc").expect("header");
        let text = value.to_str().expect("ascii");
        assert!(text.starts_with("__Host-cs_session=abc;"));
        assert!(text.contains("Secure"));
        assert!(text.contains("HttpOnly"));
        assert!(text.contains("SameSite=Lax"));
        assert!(text.contains("Path=/"));
        assert!(text.contains("Max-Age=2592000"));
        assert!(!text.contains("Domain"));
    }

    #[test]
    fn plain_http_drops_the_prefix_and_secure_flag() {
        let settings = CookieSettings::for_base_url("http://localhost:8080");
        assert_eq!(settings.session_name(), "cs_session");
        let text = settings
            .clear_session()
            .expect("header")
            .to_str()
            .expect("ascii")
            .to_owned();
        assert!(text.starts_with("cs_session=;"));
        assert!(text.contains("Max-Age=0"));
        assert!(!text.contains("Secure"));
    }

    #[test]
    fn read_cookie_finds_the_named_value_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; cs_session=token-1 ; other=x"),
        );
        assert_eq!(read_cookie(&headers, "cs_session"), Some("token-1"));
        assert_eq!(read_cookie(&headers, "missing"), None);
    }
}
