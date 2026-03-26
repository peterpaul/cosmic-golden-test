use cosmic_golden::HeadlessRenderer;
use cosmic_golden::assert_snapshot;
use cosmic_golden::assert_snapshot_rgba;

/// Covers the default (no theme) arm of `assert_snapshot!`, which must
/// produce the same baseline as the explicit `light` variant.
#[test]
fn assert_snapshot_default_theme() {
    cosmic_golden::init();
    let element: cosmic::Element<'_, ()> = cosmic::widget::text("assert_snapshot default").into();
    assert_snapshot!("assert_snapshot_default_theme", element, 320, 60);
}

/// Covers the explicit `light` arm of `assert_snapshot!`.
#[test]
fn assert_snapshot_light_explicit() {
    cosmic_golden::init();
    let element: cosmic::Element<'_, ()> = cosmic::widget::text("assert_snapshot light").into();
    assert_snapshot!("assert_snapshot_light_explicit", element, 320, 60, light);
}

/// Covers the `dark` arm of `assert_snapshot!`.
#[test]
fn assert_snapshot_dark_explicit() {
    cosmic_golden::init();
    let element: cosmic::Element<'_, ()> = cosmic::widget::text("assert_snapshot dark").into();
    assert_snapshot!("assert_snapshot_dark_explicit", element, 320, 60, dark);
}

/// Covers `assert_snapshot_rgba!` used with a manually constructed renderer.
#[test]
fn assert_snapshot_rgba_direct() {
    cosmic_golden::init();
    let mut renderer = HeadlessRenderer::new();
    let element: cosmic::Element<'_, ()> = cosmic::widget::text("assert_snapshot_rgba").into();
    let rgba = renderer.render(element, 320, 60);
    assert_snapshot_rgba!("assert_snapshot_rgba_direct", rgba, 320, 60);
}

/// Covers `assert_snapshot_rgba!` with a dark-theme renderer, exercising the
/// low-level path end-to-end with a non-default theme.
#[test]
fn assert_snapshot_rgba_dark() {
    cosmic_golden::init();
    let mut renderer = HeadlessRenderer::with_theme(cosmic::Theme::dark());
    let element: cosmic::Element<'_, ()> = cosmic::widget::text("assert_snapshot_rgba dark").into();
    let rgba = renderer.render(element, 320, 60);
    assert_snapshot_rgba!("assert_snapshot_rgba_dark", rgba, 320, 60);
}
