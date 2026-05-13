# Installation

> [!NOTE]
> The Nix flake commands below assume `nix-command` and `flakes` are
> enabled. If they aren't, add `--extra-experimental-features
> 'nix-command flakes'` to each `nix` invocation, or enable them
> permanently in your Nix configuration.

## One-shot run (no install)

With Nix:

```sh
nix run github:dk949/sptk -- '<program>'
```

Example:

```sh
printf '%s' 'foo bar baz' | nix run github:dk949/sptk -- 'S" " J","'
```

## From source (cargo)

```sh
git clone https://github.com/dk949/sptk
cd sptk
cargo install --path .
```

Or to build without installing:

```sh
cargo build --release
./target/release/sptk '<program>'
```

## From source (Nix)

Build the package:

```sh
nix build github:dk949/sptk
./result/bin/sptk '<program>'
```

Install into your profile:

```sh
nix profile install github:dk949/sptk
```

## Dev shell

A Nix dev shell with the Rust toolchain (rustc, cargo, clippy,
rustfmt, rust-analyzer):

```sh
nix develop
```
