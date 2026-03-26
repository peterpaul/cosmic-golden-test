# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-03-26

### Added

- `assert_snapshot!` now accepts an optional theme argument (`light` or `dark`),
  matching the syntax of `#[golden_test]`. Previously only the light theme was
  available without dropping down to `assert_snapshot_rgba!`.
- Unit tests for `count_differing_pixels` and `diff_image` in `snapshot.rs`.
- Integration tests covering all arms of `assert_snapshot!` and `assert_snapshot_rgba!`.
- README section documenting font handling: what `init()` guarantees, what is not
  guaranteed, and how to register additional fonts using `ctor` or `OnceLock`.

### Changed

- CI: formatting check now runs under the nightly toolchain (required for some
  rustfmt options); clippy runs as a separate parallel job.

## [0.1.0] - 2026-03-25

### Added

- `#[golden_test(width, height)]` attribute macro: converts a zero-argument
  function returning `cosmic::Element` into a `#[test]` that renders and
  compares against a PNG baseline.
- `assert_snapshot!(name, element, width, height)` macro for use inside an
  existing test function.
- `assert_snapshot_rgba!(name, rgba, width, height)` low-level macro operating
  on pre-rendered RGBA bytes.
- `HeadlessRenderer`: renders a `cosmic::Element` to raw RGBA bytes using the
  tiny-skia software backend (no display server required).
- `init()`: isolates the Cosmic Desktop font configuration and registers bundled
  Noto Sans / Noto Sans Mono fonts so rendering is identical across machines.
- Snapshot baselines stored at `<crate>/snapshots/<module>/<name>.png`;
  `.actual.png` and `.diff.png` artifacts generated on mismatch.
- `UPDATE_SNAPSHOTS=1` environment variable to regenerate baselines instead of
  comparing.
- GitHub Actions CI: build, test, and upload diff artifacts on failure.

[Unreleased]: https://github.com/peterpaul/cosmic-golden-test/compare/0.2.0...HEAD
[0.2.0]: https://github.com/peterpaul/cosmic-golden-test/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/peterpaul/cosmic-golden-test/releases/tag/0.1.0
