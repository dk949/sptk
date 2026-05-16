use crate::commands::{
    Executor, ParseCtx, Parser, Result, Value,
    parser::{int_lit, make_err, skip},
    store::StepStore,
};

/// `$N` cmd: replace current pipeline value w/ output of step N.
/// Step 0 = original input. `$` itself does not produce a new step.
pub struct PipeRef {
    idx: usize,
}

impl Parser for PipeRef {
    const CHAR: char = '$';

    fn parse<'a>(inp: &'a str, ctx: &ParseCtx) -> Result<(Self, &'a str)> {
        let inp = skip(inp);
        let Some(res) = int_lit(inp) else {
            return make_err::<_, PipeRef>(inp, "expected step index (e.g. `$1`)");
        };
        let (n, rest) = res?;
        if n >= ctx.next_step {
            return make_err::<_, PipeRef>(
                inp,
                &format!(
                    "cannot reference step {n}: only steps 0..{} exist at this point",
                    ctx.next_step
                ),
            );
        }
        Ok((PipeRef { idx: n }, rest))
    }
}

impl Executor for PipeRef {
    fn apply_pipeline(&self, store: &StepStore) -> Option<Result<Value>> {
        Some(store.get(self.idx).cloned())
    }

    fn produces_step(&self) -> bool {
        false
    }

    fn refs(&self, out: &mut Vec<usize>) {
        out.push(self.idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(n: usize) -> ParseCtx {
        ParseCtx { next_step: n }
    }

    // parser --------------------------------------------------------------

    #[test]
    fn parse_zero_at_step_one() {
        // `$0` always valid: refers to original input.
        let (p, rest) = PipeRef::parse("0", &ctx(1)).unwrap();
        assert_eq!(p.idx, 0);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_with_trailing() {
        let (p, rest) = PipeRef::parse("2 J\".\"", &ctx(3)).unwrap();
        assert_eq!(p.idx, 2);
        assert_eq!(rest, " J\".\"");
    }

    #[test]
    fn parse_skips_leading_ws() {
        let (p, rest) = PipeRef::parse("   1", &ctx(2)).unwrap();
        assert_eq!(p.idx, 1);
        assert_eq!(rest, "");
    }

    #[test]
    fn parse_missing_digits() {
        assert!(PipeRef::parse("abc", &ctx(2)).is_err());
        assert!(PipeRef::parse("", &ctx(2)).is_err());
    }

    #[test]
    fn parse_forward_ref() {
        // At step 1 (first cmd), can ref step 0 only; $1 is self-ref.
        assert!(PipeRef::parse("1", &ctx(1)).is_err());
        // $5 from step 3 is a forward ref.
        assert!(PipeRef::parse("5", &ctx(3)).is_err());
    }

    // exec ----------------------------------------------------------------

    #[test]
    fn apply_pipeline_loads_step() {
        let mut s = StepStore::new(3);
        s.set(1, Value::String("hi".into()));
        let p = PipeRef { idx: 1 };
        let out = p.apply_pipeline(&s).unwrap().unwrap();
        assert!(matches!(out, Value::String(x) if x == "hi"));
    }

    #[test]
    fn refs_reports_idx() {
        let mut v = Vec::new();
        PipeRef { idx: 4 }.refs(&mut v);
        assert_eq!(v, vec![4]);
    }

    #[test]
    fn produces_step_false() {
        assert!(!PipeRef { idx: 0 }.produces_step());
    }
}
