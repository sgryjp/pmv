<!-- markdownlint-disable MD024-->

# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- Bump [clap](https://crates.io/crates/clap) version to 4.
  This changes appearance of help message.

### Fixed

- Correct a typo in the command line help.

## 0.4.3 - 2023-11-18

### Changed

- Now pmv can be licensed under the Apache License version 2.0, or under the MIT
  license as before.

## 0.4.2 - 2022-06-26

### Fixed

- CRITICAL: If pmv swaps two files one of them will be lost. ([#20][issue20])

## 0.4.1 - 2021-03-07

### Fixed

- pmv could try to list children of _a file_ and fail ([#17][issue17])

## 0.4.0 - 2020-01-18

### Changed

- Bump version number to make a release

## 0.1.1 - 2020-01-13

### Fixed

- Disallow to move a directory to path where a file exists

[issue17]: https://github.com/sgryjp/pmv/issues/17
[issue20]: https://github.com/sgryjp/pmv/issues/20
