use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_UNESCAPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\\(.)"#).unwrap());
static DOUBLE_QUOTED_UNESCAPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\\(["\\])"#).unwrap());

pub fn parse_args_regex(input: &str) -> Vec<String> {
    let mut input = input.trim();

    let mut args = vec![];
    let re = Regex::new(r#"^"((?:[^"\\]|\\.)*)"|'([^']*)'|([^\s]+)\s*"#).unwrap();
    while let Some(caps) = re.captures(input) {
        if let Some(m) = caps.get(1) {
            let unescaped = DOUBLE_QUOTED_UNESCAPE_RE.replace_all(m.as_str(), "$1");
            args.push(unescaped.to_string());
        } else if let Some(m) = caps.get(2) {
            args.push(m.as_str().to_string());
        } else if let Some(m) = caps.get(3) {
            let unescaped = UNQUOTED_UNESCAPE_RE.replace_all(m.as_str(), "$1");
            args.push(unescaped.to_string());
        }
        input = &input[caps.get(0).unwrap().end()..];
    }
    args
}

const SQ: char = '\'';
const DQ: char = '\"';
const BS: char = '\\';

pub fn parse_args(input: &str) -> Vec<String> {
    let input = input.trim();
    let mut args = vec![];
    let (mut in_sq, mut in_dq) = (false, false);
    let mut chars = input.chars().peekable();
    let mut cur_arg = String::new();
    while let Some(c) = chars.next() {
        match (in_sq, in_dq) {
            (false, false) => match c {
                c if c.is_whitespace() => {
                    if !cur_arg.is_empty() {
                        args.push(cur_arg);
                        cur_arg = String::new()
                    }
                }
                SQ => in_sq = true,
                DQ => in_dq = true,
                BS => {
                    if let Some(next) = chars.next() {
                        cur_arg.push(next);
                    } else {
                        cur_arg.push(BS);
                    }
                }
                c => cur_arg.push(c),
            },
            (true, false) => match c {
                SQ => {
                    if let Some(&SQ) = chars.peek() {
                        chars.next();
                    } else {
                        in_sq = false;
                    }
                }
                c => cur_arg.push(c),
            },
            (false, true) => match c {
                DQ => in_dq = false,
                BS => match chars.next() {
                    Some(DQ) => cur_arg.push(DQ),
                    Some(BS) => cur_arg.push(BS),
                    Some(c) => {
                        cur_arg.push(BS);
                        cur_arg.push(c);
                    }
                    _ => cur_arg.push(BS),
                },
                c => cur_arg.push(c),
            },
            (true, true) => match c {
                DQ => in_dq = false,
                SQ => in_sq = false,
                c => cur_arg.push(c),
            },
        }
    }
    if !cur_arg.is_empty() {
        args.push(cur_arg);
    }
    args
}
