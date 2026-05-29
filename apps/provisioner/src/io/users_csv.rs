use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::domain::{ProvisionedUser, Username};
use crate::error::{HarnessError, Result};

pub fn read_users_csv(path: &Path) -> Result<Vec<ProvisionedUser>> {
    let content = fs::read_to_string(path)?;
    parse_users_csv(&content)
}

pub fn parse_users_csv(content: &str) -> Result<Vec<ProvisionedUser>> {
    let mut users = Vec::new();
    let mut seen = BTreeSet::new();

    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if users.is_empty() && trimmed.eq_ignore_ascii_case("username,password") {
            continue;
        }

        let fields: Vec<&str> = trimmed.split(',').map(str::trim).collect();
        if fields.len() != 2 {
            return Err(HarnessError::new(format!(
                "invalid users.csv line {line_number}: expected username,password"
            )));
        }

        let username = Username::parse(fields[0]).map_err(|error| {
            HarnessError::with_context(&format!("invalid username on line {line_number}"), error)
        })?;

        if fields[1].is_empty() {
            return Err(HarnessError::new(format!(
                "invalid users.csv line {line_number}: password cannot be empty"
            )));
        }

        if !seen.insert(username.as_str().to_string()) {
            return Err(HarnessError::new(format!(
                "duplicate username '{}' on line {line_number}",
                username.as_str()
            )));
        }

        users.push(ProvisionedUser::new(username, fields[1]));
    }

    Ok(users)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_users() -> Result<()> {
        let users = parse_users_csv(
            r#"
            # comment
            username,password
            alice,alice-password
            bob,bob-password
            "#,
        )?;

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username().as_str(), "alice");
        assert_eq!(users[1].username().as_str(), "bob");
        Ok(())
    }

    #[test]
    fn rejects_duplicate_users() {
        let error = parse_users_csv("alice,p1\nalice,p2")
            .err()
            .map(|value| value.to_string())
            .unwrap_or_default();

        assert!(error.contains("duplicate username"));
    }

    #[test]
    fn rejects_extra_columns() {
        assert!(parse_users_csv("alice,p1,extra").is_err());
    }
}
