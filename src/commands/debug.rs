use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::commands::{Executor, Parser, Result, Value};

static DBG_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn print(v: &Value, w: &mut impl fmt::Write) -> fmt::Result {
    match v {
        Value::String(s) => {
            write_string_lit(s, w)?;
            writeln!(w)
        }
        Value::Int(n) => writeln!(w, "{n}"),
        Value::Float(n) => writeln!(w, "{n:?}"),
        Value::List(items) => write_list(items, w, 0, false),
    }
}

fn write_string_lit(s: &str, w: &mut impl fmt::Write) -> fmt::Result {
    w.write_char('"')?;
    for c in s.chars() {
        match c {
            '\\' => w.write_str("\\\\")?,
            '"' => w.write_str("\\\"")?,
            '\0' => w.write_str("\\0")?,
            '\t' => w.write_str("\\t")?,
            '\n' => w.write_str("\\n")?,
            '\r' => w.write_str("\\r")?,
            c if (c as u32) < 0x20 || (c as u32) == 0x7F => write!(w, "\\x{:02X}", c as u32)?,
            c if c.is_control() => write!(w, "\\u{{{:04X}}}", c as u32)?,
            c => w.write_char(c)?,
        }
    }
    w.write_char('"')
}

fn write_indent(w: &mut impl fmt::Write, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        w.write_str("  ")?;
    }
    Ok(())
}

fn write_list(
    items: &[Value],
    w: &mut impl fmt::Write,
    depth: usize,
    first_inline: bool,
) -> fmt::Result {
    if items.is_empty() {
        if !first_inline {
            write_indent(w, depth)?;
        }
        return writeln!(w, "[]");
    }
    for (i, item) in items.iter().enumerate() {
        if !(first_inline && i == 0) {
            write_indent(w, depth)?;
        }
        w.write_str("- ")?;
        match item {
            Value::List(inner) => write_list(inner, w, depth + 1, true)?,
            Value::String(s) => {
                write_string_lit(s, w)?;
                writeln!(w)?;
            }
            Value::Int(n) => writeln!(w, "{n}")?,
            Value::Float(n) => writeln!(w, "{n:?}")?,
        }
    }
    Ok(())
}

pub struct Dbg;

impl Parser for Dbg {
    const CHAR: char = '_';
    fn parse(inp: &str) -> Result<(Self, &str)> {
        Ok((Dbg, inp))
    }
}

impl Executor for Dbg {
    fn apply(&self, v: &Value) -> Option<Result<Value>> {
        let n = DBG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
        let mut buf = String::new();
        let _ = writeln!(buf, "# _ #{n}");
        let _ = print(v, &mut buf);
        eprint!("{buf}");
        Some(Ok(v.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(v: &Value) -> String {
        let mut s = String::new();
        print(v, &mut s).unwrap();
        s
    }

    fn s(x: &str) -> Value {
        Value::String(x.into())
    }
    fn l(xs: Vec<Value>) -> Value {
        Value::List(xs)
    }

    // string escaping -----------------------------------------------------

    #[test]
    fn string_plain() {
        assert_eq!(render(&s("hello")), "\"hello\"\n");
    }

    #[test]
    fn string_quote_and_backslash() {
        assert_eq!(render(&s("a\"b\\c")), "\"a\\\"b\\\\c\"\n");
    }

    #[test]
    fn string_named_controls() {
        assert_eq!(render(&s("\0\t\n\r")), "\"\\0\\t\\n\\r\"\n");
    }

    #[test]
    fn string_other_ascii_control() {
        // BEL (0x07) and DEL (0x7F) -> \xNN
        assert_eq!(render(&s("\x07\x7F")), "\"\\x07\\x7F\"\n");
    }

    #[test]
    fn string_c1_control() {
        // U+0085 (NEL, a C1 control) -> \u{NNNN}
        assert_eq!(render(&s("\u{85}")), "\"\\u{0085}\"\n");
    }

    #[test]
    fn string_printable_unicode_passthrough() {
        assert_eq!(render(&s("héllo")), "\"héllo\"\n");
    }

    // scalars -------------------------------------------------------------

    #[test]
    fn int_renders() {
        assert_eq!(render(&Value::Int(42)), "42\n");
        assert_eq!(render(&Value::Int(-7)), "-7\n");
    }

    #[test]
    fn float_renders_with_dot() {
        // Debug formatting distinguishes 1.0 from 1.
        assert_eq!(render(&Value::Float(1.0)), "1.0\n");
        assert_eq!(render(&Value::Float(1.5)), "1.5\n");
    }

    // lists ---------------------------------------------------------------

    #[test]
    fn empty_list() {
        assert_eq!(render(&l(vec![])), "[]\n");
    }

    #[test]
    fn flat_mixed_list() {
        let v = l(vec![Value::Int(1), Value::Int(2), s("hello")]);
        assert_eq!(render(&v), "- 1\n- 2\n- \"hello\"\n");
    }

    #[test]
    fn nested_list_example() {
        // The exact example from the spec.
        let v = l(vec![
            Value::Int(1),
            Value::Int(2),
            s("hello"),
            l(vec![s("nested"), Value::Int(27)]),
        ]);
        let want = "- 1\n- 2\n- \"hello\"\n- - \"nested\"\n  - 27\n";
        assert_eq!(render(&v), want);
    }

    #[test]
    fn doubly_nested_list() {
        let v = l(vec![l(vec![l(vec![Value::Int(1), Value::Int(2)])])]);
        // Outer `- `, then inner `- ` (first_inline), then innermost `- ` (first_inline).
        let want = "- - - 1\n    - 2\n";
        assert_eq!(render(&v), want);
    }

    #[test]
    fn empty_inner_list() {
        let v = l(vec![Value::Int(1), l(vec![])]);
        assert_eq!(render(&v), "- 1\n- []\n");
    }

    // cmd -----------------------------------------------------------------

    #[test]
    fn dbg_parse_takes_no_arg() {
        let (_, rest) = Dbg::parse("rest").unwrap();
        assert_eq!(rest, "rest");
    }

    #[test]
    fn dbg_apply_forwards_value() {
        let v = Value::Int(7);
        let out = Dbg.apply(&v).unwrap().unwrap();
        assert!(matches!(out, Value::Int(7)));
    }
}
