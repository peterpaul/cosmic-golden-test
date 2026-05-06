//! Integration tests for overlay rendering via `render_with_events`.
//!
//! Each test drives the widget into an "overlay visible" state by feeding it
//! synthetic mouse events, then asserts the resulting screenshot matches the
//! stored baseline.  The first run (or `UPDATE_SNAPSHOTS=1`) creates the PNG;
//! subsequent runs compare pixel-by-pixel.

use std::time::Duration;

use cosmic_golden::HeadlessRenderer;
use cosmic_golden::assert_snapshot_rgba;
use cosmic_golden::cursor_move;
use cosmic_golden::left_click;

/// `combo_box` dropdown is visible after a mouse click on the text input.
///
/// `ComboBox` shows its option list as an overlay only when the embedded
/// `TextInput` is focused.  Sending a `ButtonPressed` at the widget's
/// position triggers focus and exposes the overlay, which should appear in
/// the screenshot.
#[test]
fn combo_box_dropdown_open() {
    cosmic_golden::init();

    let state = cosmic::widget::combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
    let element: cosmic::Element<'_, &'static str> =
        cosmic::widget::combo_box(&state, "Pick an option", None, |s| s).into();

    // Click in the centre of the text-input row (widget is ~40 px tall and
    // fills the full 320 px width).
    let mut renderer = HeadlessRenderer::new();
    let rgba = renderer.render_with_events(element, 320, 200, &[left_click(160.0, 20.0)]);
    assert_snapshot_rgba!("combo_box_dropdown_open", rgba, 320, 200);
}

/// `combo_box` with no click: overlay must NOT appear (baseline sanity check).
///
/// Without interaction the text input is unfocused and `combo_box` returns no
/// overlay, so the screenshot must contain only the closed widget.
#[test]
fn combo_box_closed() {
    cosmic_golden::init();

    let state = cosmic::widget::combo_box::State::new(vec!["Alpha", "Beta", "Gamma"]);
    let element: cosmic::Element<'_, &'static str> =
        cosmic::widget::combo_box(&state, "Pick an option", None, |s| s).into();

    let mut renderer = HeadlessRenderer::new();
    let rgba = renderer.render(element, 320, 200);
    assert_snapshot_rgba!("combo_box_closed", rgba, 320, 200);
}

/// `tooltip` overlay is visible when the cursor hovers over the content.
///
/// `Tooltip` transitions from `Idle` → `Open` on the first mouse event when
/// `delay` is `Duration::ZERO` and the cursor is inside the content bounds.
/// A `CursorMoved` event with the cursor positioned over the content is
/// sufficient to open the tooltip without a separate `RedrawRequested` event.
#[test]
fn tooltip_visible_on_hover() {
    cosmic_golden::init();

    let content = cosmic::widget::button::standard("Hover me");
    let tip = cosmic::widget::text("This is a tooltip");
    let element: cosmic::Element<'_, ()> =
        cosmic::widget::tooltip(content, tip, cosmic::widget::tooltip::Position::Right)
            .delay(Duration::ZERO)
            .into();

    // Move the cursor into the content area.  The button "Hover me" starts
    // at the top-left of the viewport and is roughly 120×40 px.
    let mut renderer = HeadlessRenderer::new();
    let rgba = renderer.render_with_events(element, 400, 80, &[cursor_move(60.0, 20.0)]);
    assert_snapshot_rgba!("tooltip_visible_on_hover", rgba, 400, 80);
}

/// `tooltip` without hover: the overlay must NOT appear (baseline sanity check).
#[test]
fn tooltip_hidden_without_hover() {
    cosmic_golden::init();

    let content = cosmic::widget::button::standard("Hover me");
    let tip = cosmic::widget::text("This is a tooltip");
    let element: cosmic::Element<'_, ()> =
        cosmic::widget::tooltip(content, tip, cosmic::widget::tooltip::Position::Right)
            .delay(Duration::ZERO)
            .into();

    let mut renderer = HeadlessRenderer::new();
    let rgba = renderer.render(element, 400, 80);
    assert_snapshot_rgba!("tooltip_hidden_without_hover", rgba, 400, 80);
}
