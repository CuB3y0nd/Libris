use crate::config::AppConfig;
use crate::error::{HarnessError, Result};
use crate::infra::sftpgo::SftpGoClient;
use crate::io::users_csv::read_users_csv;

pub fn run() -> Result<()> {
    let config = AppConfig::from_env()?;
    let users = read_users_csv(&config.user_file)?;

    if users.is_empty() {
        return Err(HarnessError::new("no users found in user file"));
    }

    let client = SftpGoClient::new(
        config.sftpgo.endpoint.clone(),
        config.sftpgo.admin_user.clone(),
        config.sftpgo.admin_password.clone(),
    )?;

    let token = if config.dry_run {
        String::new()
    } else {
        client.acquire_token_with_retry(config.retry_count, config.retry_delay_ms)?
    };

    for user in users {
        let key_prefix = config.s3.prefix_for(user.username());
        let payload = crate::infra::sftpgo::build_user_payload(&user, &config.s3, &key_prefix);

        if config.dry_run {
            println!(
                "dry-run: would upsert user '{}' with s3 prefix '{}'",
                user.username().as_str(),
                key_prefix
            );
            continue;
        }

        client.upsert_user(&token, user.username(), &payload)?;
        println!("upserted user '{}'", user.username().as_str());
    }

    Ok(())
}
