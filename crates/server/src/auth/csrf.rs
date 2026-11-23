//! Cross-site request protection and response headers for the `/api` routes.
//!
//! A write request (POST, PUT, PATCH, DELETE) must prove it came from this site. The browser's
//! `Origin` header is the proof when it is present. Otherwise `Sec-Fetch-Site` must say
//! `same-origin` or `none`. Otherwise the request must carry no cookie, so a browser cannot be
//! tricked into sending a session. The guard covers every `/api` route, not only the auth routes.

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};

use sha2::{Digest, Sha256};

use super::{cookies::read_cookie, errors::AuthError};

/// The largest body an auth route reads, in bytes (16 KiB).
pub const MAX_AUTH_BODY_BYTES: usize = 16 * 1024;

/// Catalogue GET paths whose successful replies hold no user-specific data for an anonymous
/// caller — signed-in callers get per-user progress on the same paths, so the cookie decides.
const PUBLIC_GET_PATHS: &[&str] = &["/api/problems", "/api/languages"];

/// How long a shared cache may keep an anonymous catalogue reply (5 minutes).
const PUBLIC_CATALOGUE_MAX_AGE: &str = "public, max-age=300";

/// Bound on the body read to compute the catalogue `ETag` (8 MiB is far beyond any catalogue page).
const MAX_CATALOGUE_BODY_BYTES: usize = 8 * 1024 * 1024;

/// The site's own origin, such as `https://counterspell.example`, taken from the public base URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebOrigin(String);

impl WebOrigin {
    /// Reads the origin (scheme, host and port) out of a base URL. A path or query is dropped.
    ///
    /// # Errors
    ///
    /// Returns the URL parser's error when `base_url` is not an absolute URL.
    pub fn from_base_url(base_url: &str) -> Result<Self, url::ParseError> {
        Ok(Self(
            url::Url::parse(base_url)?.origin().ascii_serialization(),
        ))
    }

    /// The origin text, for example `https://counterspell.example`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a request may proceed. This is the whole CSRF rule, written as a pure function.
#[must_use]
pub fn origin_allowed(
    method: &Method,
    origin: Option<&str>,
    sec_fetch_site: Option<&str>,
    has_cookie: bool,
    base_origin: &str,
) -> bool {
    if !is_write(method) {
        return true;
    }
    match (origin, sec_fetch_site) {
        // A present Origin decides alone. A wrong value is refused even if another header agrees.
        (Some(value), _) => value == base_origin,
        (None, Some(site)) => site == "same-origin" || site == "none",
        (None, None) => !has_cookie,
    }
}

fn is_write(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

fn header_text(headers: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Refuses cross-site writes before any handler runs.
pub async fn origin_guard(
    State(origin): State<Arc<WebOrigin>>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let allowed = origin_allowed(
        request.method(),
        header_text(headers, header::ORIGIN),
        header_text(headers, header::HeaderName::from_static("sec-fetch-site")),
        headers.contains_key(header::COOKIE),
        origin.as_str(),
    );
    if allowed {
        next.run(request).await
    } else {
        AuthError::CsrfOriginMismatch.into_response()
    }
}

/// Adds `Cache-Control: no-store` to every response, so account data is never cached — except
/// anonymous `GET`s on the public catalogue paths. Those carry no user-specific data, so they get
/// bounded shared caching plus a revalidating `ETag`; `Vary: Cookie` keeps a signed-in caller from
/// reusing the anonymous entry.
pub async fn cache_policy(request: Request, next: Next) -> Response {
    let headers = request.headers();
    let public_get = request.method() == Method::GET
        && PUBLIC_GET_PATHS.iter().any(|public| {
            let path = request.uri().path();
            path == *public
                || path
                    .strip_prefix(*public)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
        && read_cookie(headers, "__Host-cs_session").is_none()
        && read_cookie(headers, "cs_session").is_none();
    let validator = headers.get(header::IF_NONE_MATCH).cloned();
    let mut response = next.run(request).await;
    if !public_get || !response.status().is_success() {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        return response;
    }
    catalogue_response(response, validator).await
}

/// Stamps the public cache headers and an `ETag` on a catalogue reply, honouring `If-None-Match`.
/// The tag is weak because the edge re-encodes (compresses) the body — strong tags get rewritten
/// downstream and would never match a returning `If-None-Match`.
async fn catalogue_response(response: Response, validator: Option<HeaderValue>) -> Response {
    let (mut parts, body) = response.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_CATALOGUE_BODY_BYTES).await else {
        let mut error = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "catalogue body too large",
        )
            .into_response();
        error
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        return error;
    };
    let etag = format!("W/\"{}\"", hex::encode(Sha256::digest(&bytes)));
    parts
        .headers
        .insert(header::VARY, HeaderValue::from_static("Cookie"));
    parts.headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(PUBLIC_CATALOGUE_MAX_AGE),
    );
    parts.headers.insert(
        header::ETAG,
        HeaderValue::from_str(&etag).expect("hex etag is a valid header value"),
    );
    let revalidated = validator
        .as_ref()
        .and_then(|value| value.to_str().ok())
        .is_some_and(|field| validator_matches(field, &etag));
    if revalidated {
        parts.status = StatusCode::NOT_MODIFIED;
        parts.headers.remove(header::CONTENT_LENGTH);
        return Response::from_parts(parts, Body::empty());
    }
    Response::from_parts(parts, Body::from(bytes))
}

/// Whether an `If-None-Match` field selects this reply: `*` or any list entry equal to the tag,
/// compared weakly (the `W/` prefix is ignored on both sides, per RFC 9110).
#[must_use]
fn validator_matches(field: &str, etag: &str) -> bool {
    let ours = etag.strip_prefix("W/").unwrap_or(etag);
    split_entity_tags(field).iter().any(|candidate| {
        let candidate = candidate.trim();
        candidate == "*" || candidate.strip_prefix("W/").unwrap_or(candidate) == ours
    })
}

/// Splits a header list on commas outside quotes — an entity-tag may itself contain a comma.
#[must_use]
fn split_entity_tags(field: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (index, byte) in field.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                tags.push(&field[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    tags.push(&field[start..]);
    tags
}

/// Adds `Referrer-Policy: no-referrer` to redirects, so the next page does not see this URL.
pub async fn no_referrer_on_redirects(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    if response.status().is_redirection() {
        response.headers_mut().insert(
            header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        );
    }
    response
}

/// Applies the Origin guard and the response headers to a whole `/api` router.
pub fn apply_api_hardening<S>(router: Router<S>, origin: Arc<WebOrigin>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .layer(middleware::from_fn_with_state(origin, origin_guard))
        .layer(middleware::from_fn(cache_policy))
        .layer(middleware::from_fn(no_referrer_on_redirects))
}

/// Refuses a write with a body that is not JSON (`415`) or is over 16 KiB (`413`). The body is
/// read and measured, not judged from headers, because a request can carry a body without a
/// `Content-Length` (HTTP/2). A request with an empty body passes.
pub async fn json_body_guard(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_AUTH_BODY_BYTES).await else {
        return AuthError::BodyTooLarge.into_response();
    };
    if is_write(&parts.method) && !bytes.is_empty() && !is_json(&parts.headers) {
        return AuthError::UnsupportedMediaType.into_response();
    }
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}

fn is_json(headers: &HeaderMap) -> bool {
    header_text(headers, header::CONTENT_TYPE)
        .and_then(|value| value.split(';').next())
        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/json"))
}

/// Applies the JSON and body-size guard to the auth routes.
pub fn json_body_guard_layer<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(middleware::from_fn(json_body_guard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_base_url_reduces_to_its_origin() {
        let origin = WebOrigin::from_base_url("https://counterspell.example/some/path?q=1")
            .expect("valid URL");
        assert_eq!(origin.as_str(), "https://counterspell.example");
    }

    #[test]
    fn a_relative_base_url_is_refused() {
        assert!(WebOrigin::from_base_url("/relative").is_err());
    }

    #[test]
    fn the_rule_table_matches_the_design() {
        let base = "https://counterspell.example";
        assert!(origin_allowed(&Method::POST, Some(base), None, true, base));
        assert!(!origin_allowed(
            &Method::POST,
            Some("https://x.example"),
            Some("same-origin"),
            false,
            base
        ));
        assert!(origin_allowed(
            &Method::DELETE,
            None,
            Some("same-origin"),
            true,
            base
        ));
        assert!(!origin_allowed(
            &Method::DELETE,
            None,
            Some("cross-site"),
            false,
            base
        ));
        assert!(origin_allowed(&Method::PUT, None, None, false, base));
        assert!(!origin_allowed(&Method::PUT, None, None, true, base));
        assert!(origin_allowed(
            &Method::GET,
            Some("https://x.example"),
            None,
            true,
            base
        ));
    }
}
