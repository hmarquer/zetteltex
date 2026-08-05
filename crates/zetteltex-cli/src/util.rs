pub fn extract_title_from_tex_content(content: &str) -> Option<String> {
    let token = "\\title{";
    let start = content.find(token)? + token.len();
    let mut depth = 1usize;
    let mut i = start;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        match bytes[i] as char {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(content[start..i].trim().to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}
