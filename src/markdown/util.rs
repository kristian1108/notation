use std::fmt::Debug;
use std::iter::Peekable;

pub fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' {
            args.push(parse_quoted_string(&mut chars));
        } else {
            args.push(parse_unquoted_string(&mut chars));
        }
    }

    args
}

pub fn parse_quoted_string<I>(chars: &mut I) -> String
where
    I: Iterator<Item = char> + Debug,
{
    let mut result = String::new();
    chars.next();
    while let Some(c) = chars.next() {
        if c == '"' {
            break;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn parse_unquoted_string<I>(chars: &mut Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            break;
        } else {
            result.push(c);
            chars.next();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::markdown::util::split_args;

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_split_args() {
        let arg_string = "bin --title \"Hello, world!\"";
        let args = split_args(arg_string);
        assert_eq!(args, vec!["bin", "--title", "Hello, world!"]);
    }
}
