<!-- markdownlint-disable MD024-->
# Change Log

## [Unreleased]

### Changed

- Now pmv can be licensed under the Apache License version 2.0, or under the MIT
  license as before.

## [0.4.2] - 2022-06-26

### Fixed

- CRITICAL: If pmv swaps two files one of them will be lost. ([#20][issue20])

## [0.4.1] - 2021-03-07

### Fixed

- pmv could try to list children of *a file* and fail ([#17][issue17])

## [0.4.0] - 2020-01-18

### Changed

- Bump version number to make a release

## [0.1.1] - 2020-01-13

### Fixed

- Disallow to move a directory to path where a file exists

[issue17]: https://github.com/sgryjp/pmv/issues/17
[issue20]: https://github.com/sgryjp/pmv/issues/20
