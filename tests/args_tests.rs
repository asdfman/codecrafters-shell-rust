#[cfg(test)]
mod tests {
    use codecrafters_shell::args::parse_args;

    #[test]
    fn test_empty_input() {
        assert_eq!(parse_args(""), Vec::<String>::new());
    }

    #[test]
    fn test_single_argument() {
        assert_eq!(parse_args("foo"), vec!["foo".to_string()]);
    }

    #[test]
    fn test_multiple_arguments() {
        assert_eq!(
            parse_args("foo bar baz"),
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        assert_eq!(
            parse_args("  foo bar  "),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn test_tabs_as_whitespace() {
        assert_eq!(
            parse_args("foo\tbar"),
            vec!["foo".to_string(), "bar".to_string()]
        );
    }

    #[test]
    fn test_double_quoted_argument() {
        assert_eq!(parse_args(r#""foo bar""#), vec!["foo bar".to_string()]);
    }

    #[test]
    fn test_double_quoted_with_escaped_quote() {
        assert_eq!(parse_args(r#""foo\"bar""#), vec![r#"foo"bar"#.to_string()]);
    }

    #[test]
    fn test_double_quoted_with_escaped_backslash() {
        assert_eq!(parse_args(r#""foo\\bar""#), vec![r#"foo\bar"#.to_string()]);
    }

    #[test]
    fn test_double_quoted_with_unescaped_backslash_non_special() {
        assert_eq!(parse_args(r#""foo\bar""#), vec![r#"foo\bar"#.to_string()]);
    }

    #[test]
    fn test_double_quoted_with_single_quote_inside() {
        assert_eq!(parse_args(r#""foo'bar""#), vec!["foo'bar".to_string()]);
    }

    #[test]
    fn test_single_quoted_argument() {
        assert_eq!(parse_args(r#"'foo bar'"#), vec!["foo bar".to_string()]);
    }

    #[test]
    fn test_single_quoted_with_backslash() {
        assert_eq!(parse_args(r#"'foo\bar'"#), vec![r#"foo\bar"#.to_string()]);
    }

    #[test]
    fn test_single_quoted_with_double_quote_inside() {
        assert_eq!(parse_args(r#"'foo"bar'"#), vec![r#"foo"bar"#.to_string()]);
    }

    #[test]
    fn test_single_quoted_with_embedded_single_quote() {
        assert_eq!(parse_args(r#"'foo''bar'"#), vec!["foobar".to_string()]);
    }

    #[test]
    fn test_escaped_space_unquoted() {
        assert_eq!(parse_args(r"foo\ bar"), vec!["foo bar".to_string()]);
    }

    #[test]
    fn test_escaped_backslash_unquoted() {
        assert_eq!(parse_args(r"foo\\bar"), vec![r#"foo\bar"#.to_string()]);
    }

    #[test]
    fn test_unquoted_backslash_non_special() {
        assert_eq!(parse_args(r"foo\bar"), vec!["foobar".to_string()]);
    }

    #[test]
    fn test_mixed_quoting() {
        assert_eq!(
            parse_args(r#"foo "bar baz" 'qux'"#),
            vec!["foo".to_string(), "bar baz".to_string(), "qux".to_string()]
        );
    }

    #[test]
    fn test_nested_quotes_combination() {
        assert_eq!(
            parse_args(r#""foo 'bar' \"baz\"""#),
            vec![r#"foo 'bar' "baz""#.to_string()]
        );
    }

    #[test]
    fn test_real_cat_command_args() {
        assert_eq!(
            parse_args(r#"'./tmp/bar/f    25' './tmp/bar/f    25'"#),
            vec![
                r#"./tmp/bar/f    25"#.to_string(),
                r#"./tmp/bar/f    25"#.to_string()
            ]
        );
    }

    #[test]
    fn test_escaped_quotes_in_unquoted() {
        assert_eq!(parse_args(r#"foo\"bar"#), vec![r#"foo"bar"#.to_string()]);
    }

    #[test]
    fn test_multiple_escapes() {
        assert_eq!(parse_args(r"a\\b\ c"), vec![r"a\b c".to_string()]);
    }
}
