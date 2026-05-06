use std::borrow::Cow;
use std::sync::OnceLock;

use cosmic::Element;
use cosmic::Renderer;
use cosmic::Theme;
use cosmic::config::CosmicTk;
use cosmic::config::FontConfig;
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::advanced::graphics::text::font_system;
pub use cosmic::iced::core::Event;
use cosmic::iced::core::Font;
use cosmic::iced::core::Pixels;
use cosmic::iced::core::Size;
use cosmic::iced::core::clipboard;
use cosmic::iced::core::font;
use cosmic::iced::core::mouse;
use cosmic::iced::core::renderer;
use cosmic::iced::core::renderer::Headless;
use cosmic::iced::core::theme;
use cosmic::iced::core::widget;
use cosmic::iced::runtime::UserInterface;
use cosmic::iced::runtime::user_interface;

/// Noto Sans Regular (SIL OFL 1.1), used as the interface font in tests.
static BUNDLED_SANS: &[u8] = include_bytes!("../fonts/NotoSans-Regular.ttf");
/// Noto Sans Mono Regular (SIL OFL 1.1), used as the monospace font in tests.
static BUNDLED_MONO: &[u8] = include_bytes!("../fonts/NotoSansMono-Regular.ttf");

const BUNDLED_SANS_FAMILY: &str = "Noto Sans";
const BUNDLED_MONO_FAMILY: &str = "Noto Sans Mono";

/// Isolates the Cosmic Desktop configuration for golden tests.
///
/// Must be called **before any widget is constructed** — widget constructors
/// call `cosmic::font::default()`, which triggers `COSMIC_TK`'s `LazyLock`
/// to initialize from the real Cosmic Desktop config if it hasn't run yet.
///
/// This function does two things to make rendering environment-independent:
///
/// 1. **Config isolation** — redirects `XDG_CONFIG_HOME` to a temporary
///    directory and writes a `CosmicTk` config there that names the bundled
///    fonts. When `COSMIC_TK` later initializes it reads from this directory
///    instead of the user's real Cosmic Desktop settings.
///
/// 2. **Font registration** — loads the bundled Noto Sans and Noto Sans Mono
///    bytes into the global `FontSystem` so the family names always resolve to
///    the same bytes regardless of what system fonts are installed.
///
/// The `#[golden_test]` macro inserts this call automatically. When using
/// `assert_snapshot!` or `assert_snapshot_rgba!` directly, call this at the
/// top of the test before building any elements.
pub fn init() {
    setup_temporary_test_configuration();
}

fn setup_temporary_test_configuration() {
    static LOADED: OnceLock<()> = OnceLock::new();
    LOADED.get_or_init(|| {
        // Point XDG_CONFIG_HOME at an isolated directory so that COSMIC_TK's
        // LazyLock (which calls Config::new → dirs::config_dir() → $XDG_CONFIG_HOME)
        // reads from here rather than the real user config.
        let config_dir = std::env::temp_dir().join("cosmic-golden-isolated-config");
        // SAFETY: single-threaded at this point (OnceLock guarantees one caller).
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &config_dir) };

        // Write a CosmicTk that names the bundled fonts to the isolated
        // directory. with_custom_path creates:
        //   <config_dir>/cosmic/com.system76.CosmicTk/v1/
        let config = cosmic_config::Config::with_custom_path(
            "com.system76.CosmicTk",
            CosmicTk::VERSION,
            config_dir,
        )
        .expect("create isolated CosmicTk config");

        let mut cosmic_tk = CosmicTk::get_entry(&config).unwrap_or_default();
        cosmic_tk.interface_font = FontConfig {
            family: BUNDLED_SANS_FAMILY.to_owned(),
            weight: font::Weight::Normal,
            stretch: font::Stretch::Normal,
            style: font::Style::Normal,
        };
        cosmic_tk.monospace_font = FontConfig {
            family: BUNDLED_MONO_FAMILY.to_owned(),
            weight: font::Weight::Normal,
            stretch: font::Stretch::Normal,
            style: font::Style::Normal,
        };
        cosmic_tk
            .write_entry(&config)
            .expect("write isolated CosmicTk config");

        // Register the bundled font bytes in the global FontSystem so that
        // the family names above resolve to known bytes on every machine,
        // not to whatever version of those fonts happens to be installed.
        let mut fs = font_system().write().unwrap();
        fs.load_font(Cow::Borrowed(BUNDLED_SANS));
        fs.load_font(Cow::Borrowed(BUNDLED_MONO));
    });
}

/// The default font passed to the renderer backend.
const RENDER_FONT: Font = Font::with_name(BUNDLED_SANS_FAMILY);

/// A no-op widget operation used to trigger overlay layout computation.
///
/// `UserInterface::draw` only renders an overlay when its layout has been
/// pre-computed (stored in the UI's private `overlay` field). That field is
/// populated by `update` (event processing) or `operate`. Calling
/// `operate(&mut Noop)` before `draw` ensures any overlay that is currently
/// visible in the widget tree is actually drawn.
struct Noop;

impl widget::operation::Operation<()> for Noop {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::operation::Operation<()>)) {
        operate(self);
    }
}

/// A headless renderer that draws cosmic widgets to an in-memory RGBA buffer.
pub struct HeadlessRenderer {
    renderer: Renderer,
    theme: Theme,
}

impl HeadlessRenderer {
    /// Creates a new headless renderer using the tiny-skia software backend and the light theme.
    pub fn new() -> Self {
        let renderer = futures::executor::block_on(<Renderer as Headless>::new(
            RENDER_FONT,
            Pixels(16.0),
            Some("tiny-skia"),
        ))
        .expect("create tiny-skia headless renderer");

        Self {
            renderer,
            theme: Theme::light(),
        }
    }

    /// Creates a new headless renderer with the given theme.
    pub fn with_theme(theme: Theme) -> Self {
        let mut r = Self::new();
        r.theme = theme;
        r
    }

    /// Renders `element` into a pixel buffer of the given size.
    ///
    /// Returns raw RGBA bytes (4 bytes per pixel, row-major).
    ///
    /// Overlays that are naturally visible in the element's initial state
    /// (e.g. a tooltip that is always shown) are included in the output.
    /// For overlays that require user interaction to open (e.g. a
    /// `combo_box` dropdown), use [`render_with_events`] instead.
    ///
    /// [`render_with_events`]: HeadlessRenderer::render_with_events
    pub fn render<Message>(
        &mut self,
        element: Element<'_, Message>,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        self.render_with_events(element, width, height, &[])
    }

    /// Renders `element` into a pixel buffer after processing `events`.
    ///
    /// Each entry is `(event, cursor)`: the event to deliver and the cursor
    /// position at the time of delivery. Events are processed one at a time
    /// in order; the cursor from the last entry is used when drawing.
    ///
    /// Use this for widgets whose overlays are triggered by interaction:
    ///
    /// - **`combo_box`** — send a `ButtonPressed` at the widget's position
    ///   to focus the text input and open the dropdown.
    /// - **`pick_list`** — send a `ButtonPressed` anywhere inside the widget
    ///   to set `is_open = true` and show the menu.
    ///
    /// When `events` is empty this is identical to [`render`].
    ///
    /// [`render`]: HeadlessRenderer::render
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use cosmic_golden::{HeadlessRenderer, renderer::Event};
    /// use cosmic::iced::core::{mouse, Point};
    ///
    /// cosmic_golden::init();
    /// let state = cosmic::widget::combo_box::State::new(
    ///     vec!["Alpha", "Beta", "Gamma"],
    /// );
    /// let element: cosmic::Element<'_, &str> =
    ///     cosmic::widget::combo_box(&state, "Pick…", None, |s| s).into();
    ///
    /// let click = mouse::Cursor::Available(Point::new(100.0, 20.0));
    /// let mut r = HeadlessRenderer::new();
    /// let rgba = r.render_with_events(
    ///     element,
    ///     300, 200,
    ///     &[(Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), click)],
    /// );
    /// ```
    pub fn render_with_events<Message>(
        &mut self,
        element: Element<'_, Message>,
        width: u32,
        height: u32,
        events: &[(Event, mouse::Cursor)],
    ) -> Vec<u8> {
        let logical = Size::new(width as f32, height as f32);

        let mut ui = UserInterface::build(
            element,
            logical,
            user_interface::Cache::default(),
            &mut self.renderer,
        );

        let mut null_clipboard = clipboard::Null;
        let mut messages = Vec::<Message>::new();
        let mut cursor = mouse::Cursor::Unavailable;

        for (event, event_cursor) in events {
            cursor = *event_cursor;
            ui.update(
                std::slice::from_ref(event),
                cursor,
                &mut self.renderer,
                &mut null_clipboard,
                &mut messages,
            );
        }

        // Populate the overlay layout for any overlay that is currently
        // visible (either always-present or opened by the events above).
        // `draw` only renders overlays when this layout has been computed.
        ui.operate(&self.renderer, &mut Noop);

        let base = theme::Base::base(&self.theme);

        ui.draw(
            &mut self.renderer,
            &self.theme,
            &renderer::Style {
                icon_color: base.text_color,
                text_color: base.text_color,
                scale_factor: 1.0,
            },
            cursor,
        );

        self.renderer
            .screenshot(Size { width, height }, 1.0, base.background_color)
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}
