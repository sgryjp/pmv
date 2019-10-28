# pmv

[![Build Status](https://travis-ci.org/sgryjp/pmv.svg?branch=master)](https://travis-ci.org/sgryjp/pmv)

A command line tool to move multiple files at once, using wildcard patterns.

# Features

- Accepts wildcard patterns for selecting files to move
- Colorized terminal output
- Runs on Windows and Linux (may run on macOS too?)

# Usage

```text
USAGE:
    pmv.exe [FLAGS] <SOURCE> <DEST>

FLAGS:
    -n, --dry-run
            Do not actually move the files, just show what would be done.

    -h, --help
            Prints help information

    -V, --version
            Prints version information

    -v, --verbose
            Write verbose message.


ARGS:
    <SOURCE>
            A pattern string specifying files to move. If the pattern contains wildcard(s), multiple files
            matching to the pattern will be targeted. Supported wildcards are:

                ? ... Matches a single character
                * ... Matches zero or more characters
    <DEST>
            A pattern string specifying where to move the targeted files. If the pattern contains tokens like
            `#1` or `#2`, each of them will be replaced with a substring extracted from the targeted file path.
            Those substrings matches the wildcard patterns in SOURCE; `#1` matches the first wildcard, `#2`
            matches the second wildcard, respectively. For example, if SOURCE is `*_test.py` and DEST is
            `tests/test_#1.py`:

                Exisitng File | Destination
                ------------- | -----------------
                foo_test.py   | tests/test_foo.py
                bar_test.py   | tests/test_bar.py
                hoge_test.py  | tests/test_hoge.py
```

# Installation

## Cargo

    $ cargo install pmv

Rust 1.36.0 or newer is required to build `pmv`.

# References

- [mmv](https://ss64.com/bash/mmv.html)
