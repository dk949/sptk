# TODO

## Pipeline architecture
- [x] Core types
    - [x] Recursive `Value` (`String | List(Vec<Value>)`)
    - [x] `State` holding the current value
    - [x] `Parser` trait (associated `parse` fn + `CHAR` const)
    - [x] `Executor` trait w/ shape-aware `apply_str` + `apply_list` (default `None` = not my shape)
- [x] Zero-vtable dispatch
    - [x] `Pipeline = Vec<Command>` (no `Box<dyn>`)
    - [x] `cmds!` decl-macro: single-touch cmd registration (enum + `From` + `Executor` impl + `parse_dispatch`)
    - [x] Drop `enum_dispatch` dep (hand-rolled match arms in the macro)
- [x] Framework: `apply_to_value` w/ leaf-map + auto-promote (`String` → `List<char-as-String>`)

## Commands
- [x] `S` Split — str arg or regex arg; str → list
- [x] `J` Join — str sep; list-of-strings → string

## Parser primitives
- [x] `str_lit` (escape decoding via two-stage `delimited` + `escape_str`)
- [x] `regex_lit`
- [x] `skip` (leading whitespace)
- [x] `make_err::<_, P>` helper (uniform "Failed to parse X args at …" format)
- [x] Fix: `delimited` was iterating from the open delim → empty-string bug for symmetric delims (`"…"`, `/…/`)
- [x] Fix: `delimited` in Escape state dropped the `\` on non-special chars → `"\n"` couldn't reach `escape_str`

## CLI
- [x] Positional `PROGRAM` arg (was missing; pipeline had no way to receive a program)
- [x] `-i/-o` input/output files (default `-` = stdin/stdout)
- [x] Hard error when final value is non-String (until a render cmd lands)
- [ ] Flag to suppress automatic stripping of newline at end of input (see below)
- [ ] Better error formatting/reporting

## Text pre/post processing

- [ ] By default, remove single trailing newline from input if present
- [ ] Add automatic rendering no matter what the type of the final output is
    - [ ] This needs careful consideration for how lists will be formatted
    - [ ] Consider configurable output list format e.g. CSV, TSV
- [ ] Automatic input splitting (optional regex)
    - If `None`, no splitting, first input is just `String`
    - if `Some(Regex)`, split input on that regex, fist input is `List(String)`

## Quality
- [x] `cargo fmt` clean
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] 41 unit tests: parser primitives, Split, Join, framework dispatch (leaf-map, char-promote, no-method error)

## Documentation

- [ ] README.md
    - [ ] Broad description
    - [ ] Getting started
        - [ ] Link to install anchor in readme
        - [ ] Some basic commands
        - [ ] Link to full docs
    - [ ] Link to INSTALL.md
        - Quick note about running with nix
        - Rest is in INSTALL.md
    - [ ] Link to CONTRIBUTING.md
    - [ ] Link to LICENSE
- [ ] Full docs
    - [ ] Wiki and/or manpage? TBD
- [ ] INSTALL.md
    - [ ] `nix run` instructions
    - [ ] `cargo` build and install instructions
    - [ ] `nix` build and install instructions
- [ ] CONTRIBUTING.md
- [X] LICENSE

## CI

- [ ] Automated build + test
- [ ] Automated nix build and run
    - Make sure project can be built and smoke tested with nix
- [ ] Automated release when new tag pushed
