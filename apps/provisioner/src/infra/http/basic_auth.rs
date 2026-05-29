pub fn basic_auth_value(username: &str, password: &str) -> String {
    let credentials = format!("{username}:{password}");
    format!("Basic {}", base64_encode(credentials.as_bytes()))
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);

    let mut index = 0;
    while index < input.len() {
        let b0 = input[index];
        let b1 = input.get(index + 1).copied();
        let b2 = input.get(index + 2).copied();

        let n0 = b0 >> 2;
        let n1 = ((b0 & 0b0000_0011) << 4) | b1.unwrap_or(0) >> 4;
        let n2 = ((b1.unwrap_or(0) & 0b0000_1111) << 2) | b2.unwrap_or(0) >> 6;
        let n3 = b2.unwrap_or(0) & 0b0011_1111;

        output.push(TABLE[n0 as usize] as char);
        output.push(TABLE[n1 as usize] as char);
        output.push(if b1.is_some() {
            TABLE[n2 as usize] as char
        } else {
            '='
        });
        output.push(if b2.is_some() {
            TABLE[n3 as usize] as char
        } else {
            '='
        });

        index += 3;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_basic_auth() {
        assert_eq!(
            basic_auth_value("admin", "secret"),
            "Basic YWRtaW46c2VjcmV0"
        );
    }
}
