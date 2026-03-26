# golden

Snapshot (golden image) testing for libcosmic widgets.

Each test renders a cosmic widget tree to a PNG using the tiny-skia software renderer
(CPU-only, no display server required) and compares the result against a committed baseline.
Tests fail if any pixel differs.

## Writing a test

The preferred way is the `#[golden_test(width, height)]` attribute macro. Annotate any
zero-argument function that returns a `cosmic::Element`:

```rust
use golden::golden_test;

#[golden_test(400, 200)]
fn my_widget_light() -> cosmic::Element<'_, ()> {
    my_widget().into()
}
```

An optional third argument selects the theme — `light` (default) or `dark`:

```rust
#[golden_test(400, 200, dark)]
fn my_widget_dark() -> cosmic::Element<'_, ()> {
    my_widget().into()
}
```

The macro:
- derives the snapshot name from the **function name**
- wraps the function body in a `#[test]`
- renders with the chosen theme and compares against the stored baseline

Name the function to reflect the theme variant when testing both, so each gets
its own snapshot file (`my_widget_light.png` / `my_widget_dark.png`).

### Snapshot paths and module namespacing

Snapshots are stored inside the **snapshots directory in the crate**. The
directory structure mirrors the Rust module path of the test, so tests in
different modules never collide even when they share a function name:

```
<your-crate>/snapshots/<module>/<name>.png
```

For example, a test `view_pagination` in `read_flow::component::pagination::tests` produces:

```
read-flow/snapshots/read_flow/component/pagination/tests/view_pagination.png
```

The path is derived automatically from `module_path!()` and `env!("CARGO_MANIFEST_DIR")`
at the call site — no manual namespacing is needed.

### Using `assert_snapshot!` directly

`assert_snapshot!(name, element, width, height)` renders with the light theme and compares
against the stored baseline. An optional fifth argument selects the theme — `light` (default)
or `dark`. Use it when you need to produce multiple snapshots from a single test function,
for example with [`rstest`](https://github.com/la10736/rstest) for parameterised cases:

```rust
use golden::assert_snapshot;
use rstest::rstest;

#[rstest]
#[case("hello world",       "text_hello_world",       320, 60)]
#[case("a longer sentence", "text_a_longer_sentence",  480, 60)]
fn text_renders(
    #[case] content: &str,
    #[case] name: &str,
    #[case] width: u32,
    #[case] height: u32,
) {
    let element: cosmic::Element<'_, ()> = cosmic::widget::text(content).into();
    assert_snapshot!(name, element, width, height);
}
```

Each case produces its own baseline (`text_hello_world.png`, `text_a_longer_sentence.png`)
under the test's module path, just like `#[golden_test]` would.

The optional `dark` theme argument renders with `cosmic::Theme::dark()`:

```rust
cosmic_golden::init();
let element: cosmic::Element<'_, ()> = cosmic::widget::text("Hello").into();
assert_snapshot!("my_widget_light", element, 320, 60);
assert_snapshot!("my_widget_dark",  element, 320, 60, dark);
```

For multiple renders in a single test without the macro, construct a [`HeadlessRenderer`]
directly and use `assert_snapshot_rgba!`:

```rust
use golden::{HeadlessRenderer, assert_snapshot_rgba};

#[test]
fn my_widget_both_themes() {
    for (name, theme) in [
        ("my_widget_light", cosmic::Theme::light()),
        ("my_widget_dark",  cosmic::Theme::dark()),
    ] {
        let element: cosmic::Element<'_, ()> = my_widget().into();
        let mut r = HeadlessRenderer::with_theme(theme);
        let rgba = r.render(element, 400, 200);
        assert_snapshot_rgba!(name, rgba, 400, 200);
    }
}
```

## Font handling

### What `init()` does

`cosmic_golden::init()` makes font rendering environment-independent. It must be called
**before any widget is constructed**, because libcosmic widget constructors call
`cosmic::font::default()`, which triggers the `COSMIC_TK` global to initialize from the
user's real Cosmic Desktop config if it has not done so yet.

`init()` does two things:

1. **Config isolation** — redirects `$XDG_CONFIG_HOME` to a temporary directory and writes a
   `CosmicTk` config there naming the bundled fonts. When `COSMIC_TK` later initializes it
   reads from this directory instead of the user's real desktop settings.

2. **Font registration** — loads Noto Sans Regular and Noto Sans Mono Regular (embedded in
   the library binary) into the global `FontSystem`, so the family names always resolve to
   the same bytes regardless of what fonts are installed on the machine.

The `#[golden_test]` macro inserts this call automatically as the very first statement of the
generated test. When using `assert_snapshot!` or `assert_snapshot_rgba!` directly, call it
yourself at the top of the test before building any elements:

```rust
#[test]
fn my_test() {
    cosmic_golden::init();
    // build elements and call assert_snapshot! here
}
```

### What is guaranteed

- The **interface font** (`cosmic_tk.interface_font`) resolves to Noto Sans Regular on every
  machine, including CI runners.
- The **monospace font** (`cosmic_tk.monospace_font`) resolves to Noto Sans Mono Regular on
  every machine.
- Rendering of widgets that use only these two families is byte-for-byte identical across
  environments.

### What is not guaranteed

- **Other font families** (e.g. icon fonts, brand fonts, custom widget fonts) are not bundled.
  If a widget requests a family that is not registered, the font system falls back to whatever
  the host has installed. This can differ between a developer machine and a CI runner, causing
  spurious failures.
- **Font metrics** for families outside the bundled set are not pinned. Even if the same font
  file is installed in two places, different versions may produce different glyph outlines or
  advance widths.

### Registering additional fonts

If your widget uses an icon font or any other family beyond Noto Sans / Noto Sans Mono,
register the font bytes in the global `FontSystem`.

#### Using `ctor` (recommended)

The [`ctor`](https://crates.io/crates/ctor) crate lets you run setup code once per process
before any test starts. This is the cleanest approach: no per-test boilerplate, no `OnceLock`,
and the timing requirement of `init()` is satisfied automatically.

Add `ctor` to your dev-dependencies:

```toml
[dev-dependencies]
ctor = "0.4"
```

Then declare a single setup function in each test binary:

```rust
use ctor::ctor;
use cosmic::iced::advanced::graphics::text::font_system;
use std::borrow::Cow;

static ICON_FONT: &[u8] = include_bytes!("../fonts/MyIcons.ttf");

#[ctor]
fn test_setup() {
    cosmic_golden::init();
    font_system().write().unwrap().load_font(Cow::Borrowed(ICON_FONT));
}
```

Individual tests then need no setup at all:

```rust
#[test]
fn my_icon_widget() {
    let element: cosmic::Element<'_, ()> = my_icon_widget().into();
    cosmic_golden::assert_snapshot!("my_icon_widget", element, 320, 60);
}
```

> **Note:** each file under `tests/` is compiled as a separate binary, so a `#[ctor]`
> defined in `tests/foo.rs` does not apply to `tests/bar.rs`. Place shared setup in a
> module that each file includes with `mod setup;`, or repeat the `#[ctor]` in each file.

#### Using `OnceLock` (no extra dependency)

If you prefer not to add `ctor`, wrap the registration in a `OnceLock` and call it at
the top of each test that needs the extra font:

```rust
use cosmic::iced::advanced::graphics::text::font_system;
use std::borrow::Cow;
use std::sync::OnceLock;

static ICON_FONT: &[u8] = include_bytes!("../fonts/MyIcons.ttf");

fn setup() {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        cosmic_golden::init();
        font_system().write().unwrap().load_font(Cow::Borrowed(ICON_FONT));
    });
}

#[test]
fn my_icon_widget() {
    setup();
    let element: cosmic::Element<'_, ()> = my_icon_widget().into();
    cosmic_golden::assert_snapshot!("my_icon_widget", element, 320, 60);
}
```

## Generated files

`<crate-root>` is the root of the crate containing the test.
`<module>` is the caller's Rust module path with `::` replaced by `/`.

| File                                    | When created                       | Purpose                                           |
|-----------------------------------------|------------------------------------|---------------------------------------------------|
| `snapshots/<module>/<name>.png`         | First run, or `UPDATE_SNAPSHOTS=1` | Committed baseline                                |
| `snapshots/<module>/<name>.actual.png`  | On mismatch                        | Rendered output for inspection; **not** committed |
| `snapshots/<module>/<name>.diff.png`    | On mismatch                        | Amplified per-channel delta; **not** committed    |

On the **first run** (no baseline exists yet) the test passes and writes the baseline
automatically. Commit the new PNG to make it part of the test suite.

## When a test fails

If a test fails you will see:

```
golden: snapshot 'my_widget_dark' differs by 312 pixels.
Actual: "my-crate/snapshots/my_crate/tests/smoke_test/my_widget_dark.actual.png"
Diff:   "my-crate/snapshots/my_crate/tests/smoke_test/my_widget_dark.diff.png"
Run with UPDATE_SNAPSHOTS=1 to regenerate.
```

Three files are available for inspection:

- `snapshots/my_crate/tests/smoke_test/my_widget_dark.png` — the committed baseline
- `snapshots/my_crate/tests/smoke_test/my_widget_dark.actual.png` — what the renderer produced this run
- `snapshots/my_crate/tests/smoke_test/my_widget_dark.diff.png` — per-channel absolute difference amplified 10×;
  black means identical, bright colours indicate where and how much pixels differ

The `.actual.png` and `.diff.png` files should not be committed to git.

## Updating baselines

After verifying that the visual change is intentional:

```bash
UPDATE_SNAPSHOTS=1 cargo nextest run -p golden
```

This overwrites every baseline PNG with the current render output. Review the changed images,
then commit:

```bash
git add golden/snapshots/
git commit -m "chore: update golden image baselines"
```

To regenerate only one snapshot, run its test by name:

```bash
UPDATE_SNAPSHOTS=1 cargo nextest run -p golden -- my_widget_dark
```
