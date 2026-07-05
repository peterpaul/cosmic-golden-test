use cosmic_golden::golden_test;

/// Named icons must resolve to files from the vendored theme on platforms
/// that use freedesktop icon lookup. A failure here means the vendored
/// `icons/Cosmic` theme is missing or not found — golden tests would then
/// silently render blank icons rather than fail loudly.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn vendored_theme_resolves_named_icons() {
    cosmic_golden::init();
    let vendored_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    for name in [
        "go-first-symbolic",
        "go-previous-symbolic",
        "go-next-symbolic",
        "go-last-symbolic",
        "edit-delete-symbolic",
        "user-home-symbolic",
    ] {
        let path = cosmic::widget::icon::from_name(name)
            .path()
            .unwrap_or_else(|| panic!("icon '{name}' not found in vendored theme"));
        assert!(
            path.starts_with(&vendored_root),
            "icon '{name}' resolved outside the vendored theme: {path:?}",
        );
    }
}

#[golden_test(48, 48)]
fn icon_go_next_symbolic() -> cosmic::Element<'_, ()> {
    cosmic::widget::icon::from_name("go-next-symbolic")
        .size(32)
        .into()
}

#[golden_test(48, 48, dark)]
fn icon_go_next_symbolic_dark() -> cosmic::Element<'_, ()> {
    cosmic::widget::icon::from_name("go-next-symbolic")
        .size(32)
        .into()
}

#[golden_test(64, 48)]
fn icon_button() -> cosmic::Element<'_, ()> {
    cosmic::widget::button::icon(cosmic::widget::icon::from_name("edit-delete-symbolic").size(24))
        .into()
}
