# Nix Result Links Cleaner

A CLI program to recursively scan a specified directory for Nix result
symbolic links and optionally delete them.

## Why?

Have you ever wondered why `nix-collect-garbage` does not clean up as
much stuff as you'd think it should? Maybe all the build results of
your `nix-build` adventures are still around and keeping `/nix/store`
paths alive.

I used to run `find . -name "result*" -type l -print -delete`, but
that occasionally deletes things it shouldn't. Oops. So here we are
with a dedicated tool!

## What does it do?

This tool finds all symbolic links that are called `result` or
`result-*` and point to a Nix store path. Then it can delete them, if
you opt-in via `--delete`.

## Building

This is a Rust app without anything special. So you can do:

```console
$ cargo build --release
$ cargo run --release -- --help
```

## Contribution

Contributions are welcome! We strive for a super simple tool, so some
features might be rejected to keep it minimal.

## License

This project is licensed as GPLv3 or later.
