//! GitHub sign-in adapter against a `wiremock` stand-in for GitHub (auth T19, requirements R6.2,
//! R6.4, R6.6, R6.7, R6.9 and R6.10). No test reaches the real GitHub.

use counterspell_server::auth::{
    config::Redacted,
    email::EmailAddress,
    oauth::{github::GithubProvider, Credentials, Endpoints, OAuthProvider, ProviderId},
};
use serde_json::json;
use url::Url;
use wiremock::{
    matchers::{body_string_contains, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

const VERIFIER: &str = "verifier-0123456789-verifier-0123456789-abc";
const REDIRECT: &str = "https://counterspell.example/api/auth/oauth/github/callback";

fn provider(server: &MockServer) -> GithubProvider {
    let base = server.uri();
    GithubProvider::new(
        Credentials {
            client_id: "client-id".to_owned(),
            client_secret: Redacted::new("client-secret".to_owned()),
        },
        Endpoints {
            authorize: Url::parse(&format!("{base}/login/oauth/authorize")).expect("url"),
            token: Url::parse(&format!("{base}/login/oauth/access_token")).expect("url"),
        },
        Url::parse(&format!("{base}/user")).expect("url"),
        Url::parse(&format!("{base}/user/emails")).expect("url"),
    )
    .expect("the test client builds")
}

fn redirect() -> Url {
    Url::parse(REDIRECT).expect("valid redirect")
}

/// Mounts a token endpoint that only accepts the PKCE verifier and the client secret.
async fn mount_token(server: &MockServer, access_token: &str) {
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .and(body_string_contains(format!("code_verifier={VERIFIER}")))
        .and(body_string_contains("client_secret=client-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": access_token,
            "token_type": "bearer",
        })))
        .expect(1)
        .mount(server)
        .await;
}

async fn mount_profile(server: &MockServer, user: serde_json::Value, emails: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(user))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(emails))
        .mount(server)
        .await;
}

#[tokio::test]
async fn github_authorize_url_carries_state_challenge_and_s256() {
    let server = MockServer::start().await;
    let url = provider(&server).authorize_url("state-1", "challenge-1", &redirect());

    let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
    assert!(pairs.contains(&("state".to_owned(), "state-1".to_owned())));
    assert!(pairs.contains(&("code_challenge".to_owned(), "challenge-1".to_owned())));
    assert!(pairs.contains(&("code_challenge_method".to_owned(), "S256".to_owned())));
    assert!(pairs.contains(&("client_id".to_owned(), "client-id".to_owned())));
    assert_eq!(provider(&server).id(), ProviderId::GitHub);
}

#[tokio::test]
async fn github_token_request_carries_the_pkce_verifier_and_client_secret() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-1").await;
    mount_profile(
        &server,
        json!({ "id": 42, "login": "octo" }),
        json!([{ "email": "octo@example.co", "primary": true, "verified": true }]),
    )
    .await;

    let result = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await;

    assert!(result.is_ok(), "the exchange must send the verifier");
}

#[tokio::test]
async fn github_profile_requests_carry_the_bearer_token_and_accept_json() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-2").await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("authorization", "Bearer tok-2"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": 7 })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/emails"))
        .and(header("authorization", "Bearer tok-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(profile.provider_user_id, "7");
}

#[tokio::test]
async fn github_uses_the_numeric_id_as_the_stable_key() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-3").await;
    mount_profile(&server, json!({ "id": 42, "login": "octo" }), json!([])).await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(profile.provider_user_id, "42");
    assert_eq!(profile.label.as_deref(), Some("octo"));
    assert_eq!(profile.verified_email, None);
}

#[tokio::test]
async fn github_uses_the_primary_verified_email() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-4").await;
    mount_profile(
        &server,
        json!({ "id": 42 }),
        json!([
            { "email": "other@example.co", "primary": false, "verified": true },
            { "email": "octo@example.co", "primary": true, "verified": true },
        ]),
    )
    .await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(
        profile.verified_email,
        Some(EmailAddress::parse("octo@example.co").expect("valid"))
    );
}

#[tokio::test]
async fn github_ignores_an_unverified_or_non_primary_email() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-5").await;
    mount_profile(
        &server,
        json!({ "id": 42 }),
        json!([
            { "email": "unverified@example.co", "primary": true, "verified": false },
            { "email": "secondary@example.co", "primary": false, "verified": true },
        ]),
    )
    .await;

    let profile = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await
        .expect("profile");

    assert_eq!(profile.verified_email, None);
}

#[tokio::test]
async fn github_a_failed_exchange_is_an_error_and_is_not_echoed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .respond_with(ResponseTemplate::new(500).set_body_string("leaked-body-text"))
        .mount(&server)
        .await;

    let result = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await;

    let error = result.expect_err("a 500 from the token endpoint is a failure");
    assert!(!error.to_string().contains("leaked-body-text"));
}

#[tokio::test]
async fn github_a_malformed_profile_is_an_error() {
    let server = MockServer::start().await;
    mount_token(&server, "tok-6").await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let result = provider(&server)
        .fetch_profile("the-code", VERIFIER, &redirect())
        .await;

    assert!(result.is_err());
}
