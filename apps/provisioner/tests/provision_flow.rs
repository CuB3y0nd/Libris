use zotero_s3_webdav_provisioner::domain::{ProvisionedUser, S3Settings, Username};
use zotero_s3_webdav_provisioner::error::Result;
use zotero_s3_webdav_provisioner::infra::sftpgo::{build_user_payload, SftpGoClient};
mod support;

use support::{json_response, MockHttpServer};

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

#[test]
fn creates_missing_user() -> Result<()> {
    let server = MockHttpServer::start(vec![
        json_response(200, r#"{"access_token":"token"}"#),
        json_response(404, r#"{}"#),
        json_response(201, r#"{}"#),
    ])?;

    let client = SftpGoClient::new(server.endpoint(), "admin".into(), "secret".into())?;
    let token = client.acquire_token_with_retry(1, 0)?;

    let user = ProvisionedUser::new(Username::parse("alice")?, "alice-password");
    let s3 = S3Settings {
        bucket: "bucket".into(),
        region: "us-east-1".into(),
        endpoint: None,
        access_key: "access".into(),
        access_secret: "secret".into(),
        prefix_base: "users".into(),
        force_path_style: false,
    };
    let payload = build_user_payload(&user, &s3, "users/alice/");

    client.upsert_user(&token, user.username(), &payload)?;

    let requests = server.requests()?;
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/api/v2/token");
    assert_eq!(
        header(&requests[0].headers, "Authorization"),
        Some("Basic YWRtaW46c2VjcmV0")
    );
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/v2/users/alice");
    assert_eq!(
        header(&requests[1].headers, "Authorization"),
        Some("Bearer token")
    );
    assert_eq!(requests[2].method, "POST");
    assert_eq!(requests[2].path, "/api/v2/users");
    assert_eq!(
        header(&requests[2].headers, "Authorization"),
        Some("Bearer token")
    );
    Ok(())
}

#[test]
fn updates_existing_user() -> Result<()> {
    let server = MockHttpServer::start(vec![
        json_response(200, r#"{"access_token":"token"}"#),
        json_response(200, r#"{}"#),
        json_response(200, r#"{}"#),
    ])?;

    let client = SftpGoClient::new(server.endpoint(), "admin".into(), "secret".into())?;
    let token = client.acquire_token_with_retry(1, 0)?;

    let user = ProvisionedUser::new(Username::parse("alice")?, "alice-password");
    let s3 = S3Settings {
        bucket: "bucket".into(),
        region: "us-east-1".into(),
        endpoint: None,
        access_key: "access".into(),
        access_secret: "secret".into(),
        prefix_base: "users".into(),
        force_path_style: false,
    };
    let payload = build_user_payload(&user, &s3, "users/alice/");

    client.upsert_user(&token, user.username(), &payload)?;

    let requests = server.requests()?;
    assert_eq!(requests[2].method, "PUT");
    assert_eq!(requests[2].path, "/api/v2/users/alice");
    Ok(())
}

#[test]
fn fails_closed_when_user_lookup_errors() -> Result<()> {
    let server = MockHttpServer::start(vec![
        json_response(200, r#"{"access_token":"token"}"#),
        json_response(500, r#"{"error":"backend unavailable"}"#),
    ])?;

    let client = SftpGoClient::new(server.endpoint(), "admin".into(), "secret".into())?;
    let token = client.acquire_token_with_retry(1, 0)?;

    let user = ProvisionedUser::new(Username::parse("alice")?, "alice-password");
    let s3 = S3Settings {
        bucket: "bucket".into(),
        region: "us-east-1".into(),
        endpoint: None,
        access_key: "access".into(),
        access_secret: "secret".into(),
        prefix_base: "users".into(),
        force_path_style: false,
    };
    let payload = build_user_payload(&user, &s3, "users/alice/");

    let error = client
        .upsert_user(&token, user.username(), &payload)
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();

    assert!(error.contains("returned HTTP 500"));
    let requests = server.requests()?;
    assert_eq!(requests.len(), 2);
    Ok(())
}

#[test]
fn omits_secret_bearing_response_bodies_from_errors() -> Result<()> {
    let server = MockHttpServer::start(vec![
        json_response(200, r#"{"access_token":"token"}"#),
        json_response(404, r#"{}"#),
        json_response(
            500,
            r#"{"access_secret":"secret","password":"alice-password","access_token":"token"}"#,
        ),
    ])?;

    let client = SftpGoClient::new(server.endpoint(), "admin".into(), "secret".into())?;
    let token = client.acquire_token_with_retry(1, 0)?;

    let user = ProvisionedUser::new(Username::parse("alice")?, "alice-password");
    let s3 = S3Settings {
        bucket: "bucket".into(),
        region: "us-east-1".into(),
        endpoint: None,
        access_key: "access".into(),
        access_secret: "secret".into(),
        prefix_base: "users".into(),
        force_path_style: false,
    };
    let payload = build_user_payload(&user, &s3, "users/alice/");

    let error = client
        .upsert_user(&token, user.username(), &payload)
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();

    assert!(error.contains("returned HTTP 500"));
    assert!(!error.contains("secret"));
    assert!(!error.contains("alice-password"));
    assert!(!error.contains("token"));
    Ok(())
}
