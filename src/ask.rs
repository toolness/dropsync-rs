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

fn parse_choice<T: Copy>(input: &str, choices: &Vec<Choice<T>>) -> Option<T> {
    let lowercase_input = input.to_lowercase();

    for choice in choices {
        let lowercase_name = choice.name.to_lowercase();
        if lowercase_input == lowercase_name {
            return Some(choice.value)
        }
    }

    None
}

#[derive(Debug)]
pub struct Choice<T> {
    pub name: &'static str,
    pub value: T
}

pub fn ask_with_choices<T>(prompt: &str, choices: &Vec<Choice<T>>) -> T {
    unimplemented!()
}

#[test]
fn test_parse_yes_or_no() {
    assert_eq!(parse_yes_or_no("blarg"), None);
    assert_eq!(parse_yes_or_no("yup"), Some(true));
    assert_eq!(parse_yes_or_no("YES"), Some(true));
    assert_eq!(parse_yes_or_no("nah"), Some(false));
    assert_eq!(parse_yes_or_no("NO WAY BUDDY"), Some(false));
}

#[test]
fn test_parse_choice() {
    #[derive(Debug, PartialEq, Copy, Clone)]
    enum Boop {
        Foo,
        Bar
    }

    let choices = vec![
        Choice { name: "foo", value: Boop::Foo },
        Choice { name: "bar", value: Boop::Bar },
    ];

    assert_eq!(parse_choice("FOO", &choices), Some(Boop::Foo));
    assert_eq!(parse_choice("foo", &choices), Some(Boop::Foo));
    assert_eq!(parse_choice("", &choices), None);
    assert_eq!(parse_choice("blarg", &choices), None);
}
