use std::borrow::Cow;
use std::sync::OnceLock;

use cosmic::Element;
use cosmic::Renderer;
use cosmic::Theme;
use cosmic::config::COSMIC_TK;
use cosmic::config::CosmicTk;
use cosmic::config::FontConfig;
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

/// Bundled font faces (SIL OFL 1.1) registered by [`init`].
///
/// Noto Sans serves as the interface font and Noto Sans Mono as the monospace
/// font. Besides the regular faces, the weights and styles that libcosmic
/// widgets request are bundled so that e.g. `font::semibold()` (section
/// headers, menus), `font::bold()` (titles) and `font::light()` resolve to
/// known bytes instead of falling back to whatever the host has installed.
static BUNDLED_FONTS: &[&[u8]] = &[
    include_bytes!("../fonts/NotoSans-Light.ttf"),
    include_bytes!("../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../fonts/NotoSans-SemiBold.ttf"),
    include_bytes!("../fonts/NotoSans-Bold.ttf"),
    include_bytes!("../fonts/NotoSans-Italic.ttf"),
    include_bytes!("../fonts/NotoSans-BoldItalic.ttf"),
    include_bytes!("../fonts/NotoSansMono-Regular.ttf"),
    include_bytes!("../fonts/NotoSansMono-Bold.ttf"),
    include_bytes!("../fonts/NotoSerif-Regular.ttf"),
    include_bytes!("../fonts/NotoSerif-Bold.ttf"),
    include_bytes!("../fonts/NotoSerif-Italic.ttf"),
    include_bytes!("../fonts/NotoSerif-BoldItalic.ttf"),
];

const BUNDLED_SANS_FAMILY: &str = "Noto Sans";
const BUNDLED_MONO_FAMILY: &str = "Noto Sans Mono";
const BUNDLED_SERIF_FAMILY: &str = "Noto Serif";

/// Isolates the Cosmic Desktop configuration for golden tests.
///
/// Must be called **before any widget is constructed** — widget constructors
/// call `cosmic::font::default()`, which triggers `COSMIC_TK`'s `LazyLock`
/// to initialize from the real Cosmic Desktop config if it hasn't run yet.
///
/// This function does three things to make rendering environment-independent:
///
/// 1. **Config isolation** — redirects `XDG_CONFIG_HOME` to a temporary
///    directory (Linux-only; macOS ignores it) and overwrites the `COSMIC_TK`
///    global directly with a default configuration that names the bundled
///    fonts, replacing whatever the user's real Cosmic Desktop settings say.
///
/// 2. **Font registration** — loads the bundled Noto Sans and Noto Sans Mono
///    faces (light, regular, semibold, bold and italic variants) into the
///    global `FontSystem` so the family names
///    always resolve to the same bytes regardless of what system fonts are
///    installed — for every weight and style libcosmic widgets request.
///
/// 3. **Icon isolation** — points the freedesktop icon lookup at the Cosmic
///    icon theme vendored in this crate (`icons/Cosmic`) and hides any system
///    or user icon themes, so named icons resolve to the same SVGs on every
///    machine — including bare CI runners with no icon theme installed. On
///    macOS and Windows libcosmic never performs theme lookups; it falls back
///    to icons embedded at build time from the same `cosmic-icons` source, so
///    rendering matches across platforms.
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
        // Point XDG_CONFIG_HOME at an isolated directory so that cosmic
        // configs read via dirs::config_dir() come from here rather than the
        // real user config. dirs only honors XDG_CONFIG_HOME on Linux, so
        // this isolation is a no-op on macOS — the font configuration below
        // therefore bypasses config files entirely.
        let config_dir = std::env::temp_dir().join("cosmic-golden-isolated-config");
        // SAFETY: single-threaded at this point (OnceLock guarantees one caller).
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &config_dir) };

        // Resolve named icons from the Cosmic icon theme vendored in this
        // crate instead of whatever themes the machine has installed.
        // freedesktop-icons computes its base paths once (LazyLock), so both
        // variables must be set before the first icon lookup:
        //
        // - XDG_DATA_DIRS → this crate's root, which contains `icons/Cosmic`.
        //   Replacing (not extending) the variable hides /usr/share/icons.
        // - XDG_DATA_HOME → a non-existent directory, hiding
        //   ~/.local/share/icons.
        //
        // ~/.icons cannot be hidden this way, but XDG_DATA_DIRS is searched
        // first, so the vendored theme always shadows a user-installed one.
        //
        // SAFETY: as above.
        unsafe {
            std::env::set_var("XDG_DATA_DIRS", env!("CARGO_MANIFEST_DIR"));
            std::env::set_var(
                "XDG_DATA_HOME",
                std::env::temp_dir().join("cosmic-golden-isolated-data"),
            );
        }

        // Pin the theme name in case the test binary changed it; libcosmic
        // resolves named icons against this theme (falling back to it when a
        // different default is set).
        cosmic::icon_theme::set_default(cosmic::icon_theme::COSMIC);

        // Overwrite COSMIC_TK in place instead of writing a config file for
        // its LazyLock to pick up: file-based overrides are not portable
        // (see above), and a direct write also discards whatever desktop
        // settings the LazyLock may have read. Accessing the static here
        // triggers its initialization, before any widget queries it.
        *COSMIC_TK.write().unwrap() = CosmicTk {
            interface_font: FontConfig {
                family: BUNDLED_SANS_FAMILY.to_owned(),
                weight: font::Weight::Normal,
                stretch: font::Stretch::Normal,
                style: font::Style::Normal,
            },
            monospace_font: FontConfig {
                family: BUNDLED_MONO_FAMILY.to_owned(),
                weight: font::Weight::Normal,
                stretch: font::Stretch::Normal,
                style: font::Style::Normal,
            },
            ..CosmicTk::default()
        };

        // Register the bundled font bytes in the global FontSystem so that
        // the family names above resolve to known bytes on every machine,
        // not to whatever version of those fonts happens to be installed.
        let mut fs = font_system().write().unwrap();
        for font in BUNDLED_FONTS {
            fs.load_font(Cow::Borrowed(*font));
        }

        // Pin the generic font families to bundled ones, so text using
        // Family::Serif / SansSerif / Monospace (e.g. document viewers with a
        // serif default) renders from bundled bytes rather than whatever the
        // host maps those generics to. Cursive and fantasy have no bundled
        // equivalent; mapping them to the sans face keeps them deterministic.
        let db = fs.raw().db_mut();
        db.set_sans_serif_family(BUNDLED_SANS_FAMILY);
        db.set_serif_family(BUNDLED_SERIF_FAMILY);
        db.set_monospace_family(BUNDLED_MONO_FAMILY);
        db.set_cursive_family(BUNDLED_SANS_FAMILY);
        db.set_fantasy_family(BUNDLED_SANS_FAMILY);
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
