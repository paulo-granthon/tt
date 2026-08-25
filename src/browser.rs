pub fn translate_url(sl: &str, tl: &str, text: &str) -> String {
    format!(
        "https://translate.google.com/?sl={}&tl={}&text={}&op=translate",
        encode(sl),
        encode(tl),
        encode(text)
    )
}

pub fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
