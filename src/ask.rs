fn parse_yes_or_no(value: &str) -> Option<bool> {
    let lower = value.to_lowercase();
    if lower.starts_with("y") {
        Some(true)
    } else if lower.starts_with("n") {
        Some(false)
    } else {
        None
    }
}

pub fn ask_yes_or_no(prompt: &str) -> bool {
    loop {
        let reply = rprompt::prompt_reply_stdout(prompt).unwrap();
        if let Some(response) = parse_yes_or_no(&reply) {
            return response;
        }
    }
}

#[test]
fn test_parse_yes_or_no() {
    assert_eq!(parse_yes_or_no("blarg"), None);
    assert_eq!(parse_yes_or_no("yup"), Some(true));
    assert_eq!(parse_yes_or_no("YES"), Some(true));
    assert_eq!(parse_yes_or_no("nah"), Some(false));
    assert_eq!(parse_yes_or_no("NO WAY BUDDY"), Some(false));
}
