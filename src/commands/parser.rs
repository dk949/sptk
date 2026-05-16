use regex::Regex;

use crate::commands::{Parser, Result};

pub fn skip(inp: &str) -> &str {
    // TODO(dk949): skip comments
    inp.trim_start()
}

pub fn str_lit(inp: &str) -> Option<Result<(String, &str)>> {
    let (s, next) = match delimited(inp, '"', '"')? {
        Ok(p) => p,
        Err(e) => return Some(Err(e)),
    };
    Some(escape_str(&s).map(|x| (x, next)))
}

/// Parse a non-negative decimal integer (1+ ASCII digits).
/// Returns `None` if input doesn't start with a digit, `Some(Err)` on overflow.
pub fn int_lit(inp: &str) -> Option<Result<(usize, &str)>> {
    let end = inp.find(|c: char| !c.is_ascii_digit()).unwrap_or(inp.len());
    if end == 0 {
        return None;
    }
    let (digits, rest) = inp.split_at(end);
    Some(
        digits
            .parse::<usize>()
            .map_err(|e| e.to_string())
            .map(|n| (n, rest)),
    )
}

pub fn regex_lit(inp: &str) -> Option<Result<(Regex, &str)>> {
    let (s, next) = match delimited(inp, '/', '/')? {
        Ok(p) => p,
        Err(e) => return Some(Err(e)),
    };
    Some(Regex::new(&s).map_err(|e| e.to_string()).map(|r| (r, next)))
}

fn delimited(inp: &str, open: char, close: char) -> Option<Result<(String, &str)>> {
    if !inp.starts_with(open) {
        return None;
    }
    let body_start = open.len_utf8();
    let body = &inp[body_start..];

    enum St {
        Start,
        Escape,
    }

    let mut state = St::Start;
    let mut out = String::new();
    for (i, ch) in body.char_indices() {
        match ch {
            '\\' => match state {
                St::Start => state = St::Escape,
                St::Escape => {
                    state = St::Start;
                    out.push('\\');
                    out.push('\\');
                }
            },
            ch if ch == close => match state {
                St::Start => {
                    let rest = &inp[body_start + i + close.len_utf8()..];
                    return Some(Ok((out, rest)));
                }
                St::Escape => {
                    out.push(close);
                    state = St::Start;
                }
            },
            ch => match state {
                St::Start => out.push(ch),
                St::Escape => {
                    // Preserve `\<ch>` so escape_str (or regex engine) can decode.
                    out.push('\\');
                    out.push(ch);
                    state = St::Start;
                }
            },
        }
    }
    Some(Err(format!("Expected matching {close}")))
}

fn escape_str(inp: &str) -> Result<String> {
    // TODO(dk949): Try to do some in-place modification of `inp`
    enum St {
        Start,
        Escape,
    }
    let mut state = St::Start;
    let mut out = String::with_capacity(inp.len());
    for ch in inp.chars() {
        match state {
            St::Start => match ch {
                '\\' => state = St::Escape,
                ch => out.push(ch),
            },
            St::Escape => {
                state = St::Start;
                let decoded = match ch {
                    '\\' => '\\',
                    '0' => '\0',
                    'a' => '\x07',
                    'b' => '\x08',
                    't' => '\t',
                    'n' => '\n',
                    'v' => '\x0B',
                    'f' => '\x0C',
                    'r' => '\r',
                    ch => ch,
                };
                out.push(decoded);
            }
        }
    }
    match state {
        St::Start => Ok(out),
        St::Escape => Err("Trailing escape".into()),
    }
}

pub fn make_err<R, P: Parser>(loc: &str, msg: &str) -> Result<R> {
    let preview = loc
        .chars()
        .next()
        .map_or_else(|| "EOF".into(), |c| format!("`{c}`"));
    Err(format!(
        "Failed to parse {} args at {preview}: {msg}",
        P::CHAR
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // skip ----------------------------------------------------------------

    #[test]
    fn skip_empty() {
        assert_eq!(skip(""), "");
    }

    #[test]
    fn skip_all_ws() {
        assert_eq!(skip(" \t\n\r"), "");
    }

    #[test]
    fn skip_leading_ws() {
        assert_eq!(skip("  foo"), "foo");
    }

    #[test]
    fn skip_no_ws() {
        assert_eq!(skip("foo"), "foo");
    }

    #[test]
    fn skip_mixed_ws() {
        assert_eq!(skip(" \t\n  foo bar"), "foo bar");
    }

    // str_lit -------------------------------------------------------------

    #[test]
    fn str_lit_happy() {
        let (s, rest) = str_lit("\"foo\"").unwrap().unwrap();
        assert_eq!(s, "foo");
        assert_eq!(rest, "");
    }

    #[test]
    fn str_lit_regression_open_delim() {
        // Pre-fix bug: returned ("", "foo\"").
        let (s, rest) = str_lit("\"foo\"").unwrap().unwrap();
        assert_eq!(s, "foo");
        assert_eq!(rest, "");
    }

    #[test]
    fn str_lit_not_a_string() {
        assert!(str_lit("foo").is_none());
    }

    #[test]
    fn str_lit_trailing() {
        let (s, rest) = str_lit("\"foo\"rest").unwrap().unwrap();
        assert_eq!(s, "foo");
        assert_eq!(rest, "rest");
    }

    #[test]
    fn str_lit_escape_n() {
        let (s, _) = str_lit("\"a\\nb\"").unwrap().unwrap();
        assert_eq!(s, "a\nb");
    }

    #[test]
    fn str_lit_escape_backslash() {
        let (s, _) = str_lit("\"a\\\\b\"").unwrap().unwrap();
        assert_eq!(s, "a\\b");
    }

    #[test]
    fn str_lit_escape_quote() {
        let (s, _) = str_lit("\"a\\\"b\"").unwrap().unwrap();
        assert_eq!(s, "a\"b");
    }

    #[test]
    fn str_lit_escape_misc() {
        let (s, _) = str_lit("\"\\t\\r\\0\"").unwrap().unwrap();
        assert_eq!(s, "\t\r\0");
    }

    #[test]
    fn str_lit_unterminated() {
        assert!(str_lit("\"foo").unwrap().is_err());
    }

    // int_lit -------------------------------------------------------------

    #[test]
    fn int_lit_happy() {
        let (n, rest) = int_lit("123").unwrap().unwrap();
        assert_eq!(n, 123);
        assert_eq!(rest, "");
    }

    #[test]
    fn int_lit_trailing() {
        let (n, rest) = int_lit("42xyz").unwrap().unwrap();
        assert_eq!(n, 42);
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn int_lit_zero() {
        let (n, rest) = int_lit("0 foo").unwrap().unwrap();
        assert_eq!(n, 0);
        assert_eq!(rest, " foo");
    }

    #[test]
    fn int_lit_no_digit() {
        assert!(int_lit("abc").is_none());
    }

    #[test]
    fn int_lit_empty() {
        assert!(int_lit("").is_none());
    }

    #[test]
    fn int_lit_overflow() {
        // Way past usize::MAX.
        assert!(int_lit("99999999999999999999999999999").unwrap().is_err());
    }

    // regex_lit -----------------------------------------------------------

    #[test]
    fn regex_lit_happy() {
        let (r, rest) = regex_lit("/a+/").unwrap().unwrap();
        assert!(r.is_match("aaa"));
        assert_eq!(rest, "");
    }

    #[test]
    fn regex_lit_trailing() {
        let (r, rest) = regex_lit("/a+/xyz").unwrap().unwrap();
        assert!(r.is_match("a"));
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn regex_lit_not_a_regex() {
        assert!(regex_lit("foo").is_none());
    }

    #[test]
    fn regex_lit_invalid() {
        assert!(regex_lit("/[/").unwrap().is_err());
    }

    #[test]
    fn regex_lit_unterminated() {
        assert!(regex_lit("/abc").unwrap().is_err());
    }

    // delimited -----------------------------------------------------------

    #[test]
    fn delimited_asym() {
        let (s, rest) = delimited("[abc]rest", '[', ']').unwrap().unwrap();
        assert_eq!(s, "abc");
        assert_eq!(rest, "rest");
    }

    // escape_str ----------------------------------------------------------

    #[test]
    fn escape_str_plain() {
        assert_eq!(escape_str("foo").unwrap(), "foo");
    }

    #[test]
    fn escape_str_all_escapes() {
        assert_eq!(
            escape_str("\\0\\a\\b\\t\\n\\v\\f\\r\\\\").unwrap(),
            "\0\x07\x08\t\n\x0B\x0C\r\\"
        );
    }

    #[test]
    fn escape_str_unknown_passthrough() {
        // Unknown escapes drop the backslash and keep the char.
        assert_eq!(escape_str("\\x").unwrap(), "x");
    }

    #[test]
    fn escape_str_trailing_backslash() {
        assert!(escape_str("foo\\").is_err());
    }
}
