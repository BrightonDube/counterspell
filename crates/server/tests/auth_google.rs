//! Google sign-in adapter against a `wiremock` stand-in for Google (auth T20, requirements R6.2,
//! R6.4, R6.6, R6.7, R6.9 and R6.10). No test reaches the real Google.

use counterspell_server::auth::{
    config::Redacted,
    email::EmailAddress,
    oauth::{google::GoogleProvider, Credentials, Endpoints, OAuthProvider, ProviderId},
};
use serde_json::json;
use url::Url;
use wiremock::{
    matchers::{body_string_contains, method, path},
    Mock, MockServer, ResponseTemplate,
};

const VERIFIER: &str = "verifier-0123456789-verifier-0123456789-abc";
const REDIRECT: &str = "https://counterspell.example/api/auth/oauth/google/callback";

fn provider(server: &MockServer) -> GoogleProvider {
    let base = server.uri();
    GoogleProvider::new(
        Credentials {
            client_id: "google-client".to_owned(),
            client_secret: Redacted::new("google-secret".to_owned()),
        },
        Endpoints {
            authorize: Url::parse(&format!("{base}/auth")).expect("url"),
            token: Url::parse(&format!("{base}/token")).expect("url"),
        },
        Url::parse(&format!("{base}/userinfo")).expect("url"),
    )
    .expect("the test client builds")
}

fn redirect() -> Url {
    Url::parse(REDIRECT).expect("valid redirect")
}

/// Mounts a token endpoint that needs the PKCE verifier, and a userinfo answer.
async fn mount(server: &MockServer, userinfo: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains(format!("code_verifier={VERIFIER}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "g-tok",
            "token_type": "Bearer",
        })))
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/userinfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(userinfo))
        .mount(server)
        .await;
}

#[tokio::test]
async fn google_authorize_url_asks_for_openid_email_and_profile_with_pkce() {
    let server = MockServer::start().await;
    let url = provider(&server).authorize_url("state-g", "challenge-g", &redirect());

    let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
    assert!(pairs.contains(&("scope".to_owned(), "openid email profile".to_owned())));
    assert!(pairs.contains(&("code_challenge_method".to_owned(), "S256".to_owned())));
    assert_eq!(provider(&server).id(), ProviderId::Google);
}

#[tokio::test]
async fn google_token_request_carries_the_pkce_verifier() {
    let server = MockServer::start().await;
    mount(
        &server,
        json!({ "sub": "1234", "email": "g@example.co", "email_verified": true }),
    )
    .await;

    let result = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await;

    assert!(result.is_ok(), "the exchange must send the verifier");
}

#[tokio::test]
async fn google_uses_sub_as_the_stable_key_and_a_verified_email() {
    let server = MockServer::start().await;
    mount(
        &server,
        json!({ "sub": "1234", "name": "Thandi", "email": "g@example.co", "email_verified": true }),
    )
    .await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(profile.provider_user_id, "1234");
    assert_eq!(profile.label.as_deref(), Some("Thandi"));
    assert_eq!(
        profile.verified_email,
        Some(EmailAddress::parse("g@example.co").expect("valid"))
    );
}

#[tokio::test]
async fn google_an_unverified_email_is_not_used() {
    let server = MockServer::start().await;
    mount(
        &server,
        json!({ "sub": "1234", "email": "g@example.co", "email_verified": false }),
    )
    .await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(profile.verified_email, None);
}

#[tokio::test]
async fn google_a_profile_without_sub_is_an_error() {
    let server = MockServer::start().await;
    mount(
        &server,
        json!({ "email": "g@example.co", "email_verified": true }),
    )
    .await;

    let result = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn google_an_error_body_is_not_echoed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string("leaked-google-body"))
        .mount(&server)
        .await;

    let error = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect_err("a 400 from the token endpoint is a failure");

    assert!(!error.to_string().contains("leaked-google-body"));
}
