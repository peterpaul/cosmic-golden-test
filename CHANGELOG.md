# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Font weights and styles: besides Noto Sans Regular and Noto Sans Mono
  Regular, `init()` now registers Noto Sans Light, SemiBold, Bold, Italic and
  Bold Italic, and Noto Sans Mono Bold (all v2.008/v2.007 hinted, from
  googlefonts/noto-fonts commit `ffebf8c`). Widgets using `font::light()`,
  `font::semibold()` (headings, menus), `font::bold()` (titles) or italic
  spans now render identically on every machine instead of falling back to
  host-installed fonts.
- Icon isolation: `init()` now points the freedesktop icon lookup at a vendored
  copy of the Cosmic icon theme (`icons/Cosmic`, from cosmic-icons commit
  `5252095`) and hides system and user icon themes. Named symbolic icons render
  identically on every machine, including CI runners with no icon theme
  installed. On macOS/Windows libcosmic already uses embedded icons from the
  same source, so rendering matches across platforms.

## [0.5.0] - 2026-07-04

### Fixed

- Font configuration now works on macOS. `init()` previously wrote the `CosmicTk`
  font override to a config file under `$XDG_CONFIG_HOME`, which cosmic-config
  (via `dirs::config_dir()`) only honors on Linux; on macOS the override was
  silently ignored, widgets fell back to the default "Open Sans" family — which
  is not bundled — and every snapshot differed from its baseline. The bundled
  font families are now written directly to the `COSMIC_TK` global, which works
  on every platform and also takes effect when a real desktop config was
  already loaded.

## [0.4.0] - 2026-05-06

### Added

- `HeadlessRenderer::render_with_events(element, width, height, events)` renders a
  widget after processing a sequence of synthetic `(Event, mouse::Cursor)` pairs.
  Use this to capture widgets whose overlays are triggered by interaction, such as a
  `combo_box` with its dropdown open or a `tooltip` visible on hover.
- `render` now draws overlays that are naturally visible in the element's initial
  state (e.g. always-shown tooltips) by running a no-op widget operation before
  drawing to ensure the overlay layout is pre-computed.
- `cosmic_golden::Event` re-export (`cosmic::iced::core::Event`) so callers do not
  need a direct `iced` dependency when building event sequences.
- `cosmic_golden::events` module — convenience functions that build
  `(Event, mouse::Cursor)` pairs ready for use with `render_with_events`:
  `left_click(x, y)`, `right_click(x, y)`, `left_release(x, y)`,
  `cursor_move(x, y)`, `scroll(x, y, delta_x, delta_y)`.
  All five are re-exported at the crate root.
- `#[golden_test]` now accepts an optional `events = [expr, ...]` argument.
  When present, the generated test calls `render_with_events` instead of `render`,
  allowing overlays triggered by interaction to be captured without dropping down
  to `assert_snapshot_rgba!`.
- Integration tests covering `render_with_events`: `combo_box` dropdown open/closed
  and `tooltip` visible/hidden, each with a committed PNG baseline.

## [0.3.0] - 2026-04-15

### Changed

- Update libcosmic

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

[Unreleased]: https://github.com/peterpaul/cosmic-golden-test/compare/0.5.0...HEAD
[0.5.0]: https://github.com/peterpaul/cosmic-golden-test/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/peterpaul/cosmic-golden-test/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/peterpaul/cosmic-golden-test/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/peterpaul/cosmic-golden-test/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/peterpaul/cosmic-golden-test/releases/tag/0.1.0
