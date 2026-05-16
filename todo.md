# TODO

## Pipeline architecture
- [x] Core types
    - [x] Recursive `Value` (`String | List(Vec<Value>)`)
    - [x] `State` holding the current value
    - [x] `Parser` trait (associated `parse` fn + `CHAR` const)
    - [x] `Executor` trait w/ shape-aware `apply_str` + `apply_list`
          (default `None` = not my shape)
- [x] Zero-vtable dispatch
    - [x] `Pipeline = Vec<Command>` (no `Box<dyn>`)
    - [x] `cmds!` decl-macro: single-touch cmd registration
          (enum + `From` + `Executor` impl + `parse_dispatch`)
    - [x] Drop `enum_dispatch` dep (hand-rolled match arms in the macro)
- [x] Framework: `apply_to_value` w/ leaf-map + auto-promote
      (`String` → `List<char-as-String>`)

## Commands
- [x] `S` Split: str arg or regex arg; str → list
- [x] `J` Join: str sep; list-of-strings → string
- [ ] `n` Number: parse str → Int/Float
    - [x] Sketch: f/i/h/o/b strict, F/I/H/O/B partial (zero on no match);
          optional base prefix; signs only for f/i
    - [ ] Decide rendering / further-cmd interaction for Int/Float values
- [x] `_` Debug: print value to stderr, forward unchanged
    - Adds `Executor::apply(&Value)` hook for whole-value passthrough
    - YAML-style list format; string-escape `\\` `"` named controls,
      `\xNN` for other ASCII controls + DEL, `\u{NNNN}` for C1 controls

## Pipeline output references
- [x] `$N` pipeline-subst cmd
    - `$` does NOT count as a step itself; step numbering only counts
      step-producing cmds (1-indexed, `$0` = original input)
    - [x] `ParseCtx { next_step }` (read-only); extend `Parser::parse` sig;
          update macro + existing cmds to accept (and ignore) ctx
    - [x] `int_lit` parser primitive + tests
    - [x] `Executor::produces_step` (default `true`); `$` overrides `false`
    - [x] `StepStore` (only referenced slots populated)
    - [x] `Command::refs` via macro; static pre-scan in runner
    - [x] `Executor::apply_pipeline` hook
    - [x] `$` cmd: parser, executor, registry
    - [x] Forward-ref + too-large-ref parse errors (`n < ctx.next_step`)
    - [x] E2E + unit tests
- [x] Refactor existing coercion sites into `commands/coerce.rs`
    - [x] Move `int_to_float` / `int_to_str` / `float_to_int` /
          `float_to_str` / `string_to_chars` out of `apply_to_value`
    - [x] `apply_to_value` calls the new helpers
- [x] `${N}` arg substitution
    - Brace form required (`${N}`); bare `$N` only as Phase 1 cmd.
      No interpolation inside `"..."`. Refs must be int literals
      (no nested `${${1}}`, no `$ ${1}`).
    - [x] `Arg<T>` enum
    - [x] `pipeline_ref` parser primitive (`${digits}`) + tests
    - [x] Value-level coerce helpers in `coerce.rs`
          (`to_string` / `to_int` / `to_float` / `to_regex`)
    - [x] `Executor::resolve` hook (+ macro plumbing)
    - [x] Uplift `Join` sep to `Arg<String>`
    - [x] Uplift `Split` `On` variants to `Arg<_>`
        - Data shape ready; parser still rejects bare `${N}` (no
          string-vs-regex disambiguator yet)
    - [ ] Disambiguation syntax for Split's `${N}` (string vs regex)
    - [x] Unit + E2E tests
- [ ] Warn when `$N` is the final cmd (or only `$N`s follow)
    - `x 123 y 123 $1`: `y`'s output is discarded; only useful side-effect
      cmds could justify this. Until impure cmds exist, print a warning.
    - Revisit once impure cmds (side effects) land.

## Parser primitives
- [x] `str_lit` (escape decoding via two-stage `delimited` + `escape_str`)
- [x] `regex_lit`
- [x] `skip` (leading whitespace)
- [x] `make_err::<_, P>` helper (uniform "Failed to parse X args at …" format)
- [x] Fix: `delimited` was iterating from the open delim, causing an
      empty-string bug for symmetric delims (`"…"`, `/…/`)
- [x] Fix: `delimited` in Escape state dropped the `\` on non-special
      chars, so `"\n"` couldn't reach `escape_str`

## CLI
- [x] Positional `PROGRAM` arg (was missing; pipeline had no way to
      receive a program)
- [x] `-i/-o` input/output files (default `-` = stdin/stdout)
- [x] Hard error when final value is non-String (until a render cmd lands)
- [x] Flag to suppress automatic stripping of newline at end of input (see below)
- [x] `-d`/`--debug` flag: debug-print final pipeline value (any shape)
- [ ] Better error formatting/reporting

## Text pre/post processing

- [x] By default, remove single trailing newline from input if present
    - Strips `\n` or `\r\n`; appends single `\n` on output (mirror).
      `--raw` opts out of both.
- [ ] Add automatic rendering no matter what the type of the final output is
    - [ ] This needs careful consideration for how lists will be formatted
    - [ ] Consider configurable output list format e.g. CSV, TSV
- [ ] Automatic input splitting (optional regex)
    - If `None`, no splitting, first input is just `String`
    - if `Some(Regex)`, split input on that regex, fist input is `List(String)`

## Quality
- [x] `cargo fmt` clean
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] 41 unit tests: parser primitives, Split, Join, framework dispatch
      (leaf-map, char-promote, no-method error)

## Documentation

- [x] README.md
    - [x] Broad description
    - [x] Getting started
        - [x] Link to install anchor in readme
        - [x] Some basic commands
        - [ ] Link to full docs
    - [x] Link to INSTALL.md
        - Quick note about running with nix
        - Rest is in INSTALL.md
    - [x] Link to CONTRIBUTING.md
    - [x] Link to LICENSE
- [ ] Full docs
    - [ ] Wiki and/or manpage? TBD
- [x] INSTALL.md
    - [x] `nix run` instructions
    - [x] `cargo` build and install instructions
    - [x] `nix` build and install instructions
- [x] CONTRIBUTING.md
- [X] LICENSE

## CI

- [x] Automated build + test
- [x] Automated nix build and run
    - Make sure project can be built and smoke tested with nix
- [x] Automated release when new tag pushed
- [x] Fix nix caching
    - Magic nix cache seems broken
