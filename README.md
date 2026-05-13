# sptk

**String Processing Tool Kit**. A small CLI language for string
manipulation, inspired by awk and sed.

## Quickstart

See [INSTALL.md](INSTALL.md) for install options. The fastest is
`nix run github:dk949/sptk -- '<program>'`.

```sh
# Split on a literal, join with a comma:
$ printf '%s' 'foo bar baz' | sptk 'S" " J","'
foo,bar,baz

# Split on a regex, then re-join:
$ printf '%s' 'a  b   c' | sptk 'S/\s+/ J"-"'
a-b-c
```

By default sptk reads from stdin and writes to stdout. Use
`-i FILE` / `-o FILE` to redirect.

> [!NOTE]
> The output must currently be a string. If a program ends with a
> command that produces a list, finish it with a `J` to join the
> parts back into a single string.

## Commands

| Char | Name  | Args                 | Description                           |
| ---- | ----- | -------------------- | ------------------------------------- |
| `S`  | Split | `"sep"` or `/regex/` | Split a string into a list            |
| `J`  | Join  | `"sep"`              | Join a list of strings with separator |

## How it works

Commands are parsed left-to-right and chained: each consumes its own
arguments and pipes its output to the next. A program is a sequence
of single-letter commands.

When a command runs on something other than the shape it expects (for
example applying `S` to a list, or applying `J` to a bare string),
sptk adapts. `S` runs on each element of the list. `J` treats a bare
string as its sequence of characters and joins them with the chosen
separator:

```sh
$ printf '%s' 'hello' | sptk 'J"-"'
h-e-l-l-o
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
