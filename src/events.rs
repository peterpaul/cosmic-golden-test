use cosmic::iced::core::Point;
use cosmic::iced::core::mouse;

use crate::renderer::Event;

/// Returns a left-button press event with the cursor at `(x, y)`.
///
/// Use this to focus a `combo_box`, activate a `button`, or open a `pick_list`.
pub fn left_click(x: f32, y: f32) -> (Event, mouse::Cursor) {
    button_press(x, y, mouse::Button::Left)
}

/// Returns a right-button press event with the cursor at `(x, y)`.
pub fn right_click(x: f32, y: f32) -> (Event, mouse::Cursor) {
    button_press(x, y, mouse::Button::Right)
}

/// Returns a left-button release event with the cursor at `(x, y)`.
pub fn left_release(x: f32, y: f32) -> (Event, mouse::Cursor) {
    button_release(x, y, mouse::Button::Left)
}

/// Returns a cursor-moved event to position `(x, y)`.
///
/// Sufficient to open a `tooltip` with `delay(Duration::ZERO)` when the cursor
/// lands inside the content bounds.
pub fn cursor_move(x: f32, y: f32) -> (Event, mouse::Cursor) {
    let pos = Point::new(x, y);
    (
        Event::Mouse(mouse::Event::CursorMoved { position: pos }),
        mouse::Cursor::Available(pos),
    )
}

/// Returns a pixel-based scroll event with the cursor at `(x, y)`.
///
/// Positive `delta_y` scrolls down; positive `delta_x` scrolls right.
pub fn scroll(x: f32, y: f32, delta_x: f32, delta_y: f32) -> (Event, mouse::Cursor) {
    (
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels {
                x: delta_x,
                y: delta_y,
            },
        }),
        mouse::Cursor::Available(Point::new(x, y)),
    )
}

fn button_press(x: f32, y: f32, button: mouse::Button) -> (Event, mouse::Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonPressed(button)),
        mouse::Cursor::Available(Point::new(x, y)),
    )
}

fn button_release(x: f32, y: f32, button: mouse::Button) -> (Event, mouse::Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonReleased(button)),
        mouse::Cursor::Available(Point::new(x, y)),
    )
}
