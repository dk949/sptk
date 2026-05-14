use std::{
    fs,
    io::{self, Read, Write},
};

pub fn read_file(path: &str) -> io::Result<String> {
    if path == "-" {
        let mut stdin = io::stdin();
        let mut out = String::new();
        stdin.read_to_string(&mut out)?;
        Ok(out)
    } else {
        fs::read_to_string(path)
    }
}

pub fn write_file(path: &str, data: &str) -> io::Result<()> {
    if path == "-" {
        let mut stdout = io::stdout();
        stdout.write_all(data.as_bytes())
    } else {
        fs::write(path, data)
    }
}

/// Strip one trailing line ending (`\n` or `\r\n`) if present.
pub fn strip_trailing_newline(s: &str) -> &str {
    match s.strip_suffix('\n') {
        Some(t) => t.strip_suffix('\r').unwrap_or(t),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_lf() {
        assert_eq!(strip_trailing_newline("foo\n"), "foo");
    }

    #[test]
    fn strip_crlf() {
        assert_eq!(strip_trailing_newline("foo\r\n"), "foo");
    }

    #[test]
    fn strip_only_one() {
        assert_eq!(strip_trailing_newline("foo\n\n"), "foo\n");
        assert_eq!(strip_trailing_newline("foo\r\n\r\n"), "foo\r\n");
    }

    #[test]
    fn strip_none_when_absent() {
        assert_eq!(strip_trailing_newline("foo"), "foo");
        assert_eq!(strip_trailing_newline(""), "");
    }

    #[test]
    fn strip_lone_cr_not_touched() {
        // A bare `\r` at end is not a line ending we recognise; leave it.
        assert_eq!(strip_trailing_newline("foo\r"), "foo\r");
    }

    #[test]
    fn strip_newline_only_input() {
        assert_eq!(strip_trailing_newline("\n"), "");
        assert_eq!(strip_trailing_newline("\r\n"), "");
    }
}
