# pmv

[![CI](https://github.com/sgryjp/pmv/actions/workflows/ci.yml/badge.svg)][ci-status]
[![Latest version](https://img.shields.io/crates/v/pmv)](https://crates.io/crates/pmv)

A command line tool to move multiple files at once, using wildcard patterns.

## Features

- Accepts wildcard patterns for selecting files to move
- Colorized terminal output
- Runs on Windows, Linux and macOS

## Usage

```text
USAGE:
    pmv.exe [FLAGS] <SOURCE> <DEST>

FLAGS:
    -n, --dry-run
            Does not move files but just shows what would be done

    -h, --help
            Prints help information

    -i, --interactive
            Prompts before moving an each file

    -V, --version
            Prints version information

    -v, --verbose
            Writes verbose message


ARGS:
    <SOURCE>
            A pattern string specifying files to move. If the
            pattern contains wildcard(s), multiple files matching
            to the pattern will be targeted. Supported wildcards
            are:

                ? ... Matches a single character
                * ... Matches zero or more characters
    <DEST>
            A pattern string specifying where to move the targeted
            files. If the pattern contains tokens like `#1` or
            `#2`, each of them will be replaced with a substring
            extracted from the targeted file path. Those substrings
            matches the wildcard patterns in SOURCE; `#1` matches
            the first wildcard, `#2` matches the second,
            respectively. For example, if SOURCE is `*_test.py` and
            DEST is `tests/test_#1.py`:

                Exisitng File | Destination
                ------------- | -----------------
                foo_test.py   | tests/test_foo.py
                bar_test.py   | tests/test_bar.py
                hoge_test.py  | tests/test_hoge.py
```

## Installation

### Cargo

```shell
cargo install pmv
```

Rust 1.36.0 or newer is required to build `pmv`.

## License

pmv is licensed under the [Apache License version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT), at your option.

## References

- [mmv](https://ss64.com/bash/mmv.html)

[ci-status]: https://github.com/sgryjp/pmv/actions/workflows/ci.yml
