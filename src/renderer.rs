use std::sync::OnceLock;

use cosmic::Element;
use cosmic::Renderer;
use cosmic::Theme;
use cosmic::config::CosmicTk;
use cosmic::cosmic_config;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced_core::Font;
use cosmic::iced_core::Pixels;
use cosmic::iced_core::Size;
use cosmic::iced_core::mouse;
use cosmic::iced_core::renderer;
use cosmic::iced_core::renderer::Headless;
use cosmic::iced_core::theme;
use cosmic::iced_runtime::UserInterface;
use cosmic::iced_runtime::user_interface;


/// Isolates the Cosmic Desktop configuration for golden tests.
///
/// Must be called **before any widget is constructed** — widget constructors
/// call `cosmic::font::default()`, which triggers `COSMIC_TK`'s `LazyLock`
/// to initialize from the real Cosmic Desktop config if it hasn't run yet.
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

        // Write a default CosmicTk to the isolated directory.
        // with_custom_path creates: <config_dir>/cosmic/com.system76.CosmicTk/v1/
        let config = cosmic_config::Config::with_custom_path(
            "com.system76.CosmicTk",
            CosmicTk::VERSION,
            config_dir,
        )
        .expect("create isolated CosmicTk config");

        let cosmic_tk = CosmicTk::get_entry(&config).unwrap_or_default();
        cosmic_tk
            .write_entry(&config)
            .expect("write isolated CosmicTk config");
    });
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
            Font::DEFAULT,
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
    pub fn render<Message>(
        &mut self,
        element: Element<'_, Message>,
        width: u32,
        height: u32,
    ) -> Vec<u8> {
        let logical = Size::new(width as f32, height as f32);

        let mut ui = UserInterface::build(
            element,
            logical,
            user_interface::Cache::default(),
            &mut self.renderer,
        );

        let base = theme::Base::base(&self.theme);

        ui.draw(
            &mut self.renderer,
            &self.theme,
            &renderer::Style {
                icon_color: base.text_color,
                text_color: base.text_color,
                scale_factor: 1.0,
            },
            mouse::Cursor::Unavailable,
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
