use zotero_s3_webdav_provisioner::domain::{ProvisionedUser, S3Settings, Username};
use zotero_s3_webdav_provisioner::error::Result;
use zotero_s3_webdav_provisioner::infra::sftpgo::{build_user_payload, SftpGoClient};
mod support;

use support::{json_response, MockHttpServer};

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
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/v2/users/alice");
    assert_eq!(requests[2].method, "POST");
    assert_eq!(requests[2].path, "/api/v2/users");
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
