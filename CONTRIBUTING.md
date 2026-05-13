# Contributing

## Workflow

1. Open an issue first for non-trivial changes; happy to discuss design
   before code lands.
2. Fork, branch, PR against `trunk`.
3. Before opening the PR, make sure these are all clean:

   ```sh
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) format:
`type(scope): subject`.

Examples:

- `feat(commands): add U cmd for upcase`
- `fix(parser): handle empty regex literal`
- `refactor(commands): collapse Split/Join shared parsing`
- `chore: bump regex to 1.13`

Common types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`.

## Adding a new command

Each command lives in `src/commands/<name>.rs` with two impls:

- `Parser`: a `const CHAR` (the single-letter trigger) and a
  `fn parse(inp: &str) -> Result<(Self, &str)>` that consumes the
  argument tokens from the input and returns the unconsumed tail.
- `Executor`: override `apply_str` and/or `apply_list` as appropriate.
  Each returns `Option<Result<Value>>`; default `None` means "not my
  shape", and the framework will leaf-map or auto-promote as needed.

Register the command in `src/commands/mod.rs` by appending its type
ident to the `cmds!` invocation, e.g. `cmds! { Split, Join, NewCmd }`.
The macro generates the `Command` enum variant, the `From` impl, the
`Executor` impl, and the parse-dispatch arm.

> [!TIP]
> Parser primitives in `src/commands/parser.rs` (`str_lit`,
> `regex_lit`, `skip`, `make_err`) are reusable. Reach for them
> before writing argument parsing by hand.

Add unit tests in the same file under `#[cfg(test)] mod tests`.

## License

By contributing, you agree your contributions are licensed under the
[MIT License](LICENSE).
