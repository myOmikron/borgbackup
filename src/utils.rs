/// Helper function for shell escaping
///
/// It will try to quote the entire string and escape characters within that need escaping
pub(crate) fn shell_escape(input: &str) -> String {
    let mut s = String::new();
    s.push('\'');
    for c in input.chars() {
        match c {
            '\'' => s.push_str(r#"\'"#),
            '\\' => s.push_str(r#"\\"#),
            _ => s.push(c),
        }
    }
    s.push('\'');
    s
}
