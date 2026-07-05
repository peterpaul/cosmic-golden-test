use cosmic::iced::core::font;
use cosmic_golden::HeadlessRenderer;
use cosmic_golden::golden_test;

fn styled_text(weight: font::Weight, style: font::Style) -> cosmic::Element<'static, ()> {
    let font = cosmic::iced::core::Font {
        weight,
        style,
        ..cosmic::font::default()
    };
    cosmic::widget::text("Grumpy wizards 123").font(font).into()
}

/// Every bundled variant must render differently from every other one.
/// If a weight or style is not registered, the font system silently falls
/// back to another face and two renders become identical — golden tests
/// alone would then bless the fallback.
#[test]
fn bundled_variants_render_distinctly() {
    cosmic_golden::init();
    let variants = [
        ("light", font::Weight::Light, font::Style::Normal),
        ("regular", font::Weight::Normal, font::Style::Normal),
        ("semibold", font::Weight::Semibold, font::Style::Normal),
        ("bold", font::Weight::Bold, font::Style::Normal),
        ("italic", font::Weight::Normal, font::Style::Italic),
        ("bold_italic", font::Weight::Bold, font::Style::Italic),
    ];
    let mut renderer = HeadlessRenderer::new();
    let renders: Vec<(&str, Vec<u8>)> = variants
        .into_iter()
        .map(|(name, weight, style)| (name, renderer.render(styled_text(weight, style), 220, 40)))
        .collect();
    for (i, (name_a, rgba_a)) in renders.iter().enumerate() {
        for (name_b, rgba_b) in renders.iter().skip(i + 1) {
            assert_ne!(
                rgba_a, rgba_b,
                "'{name_a}' and '{name_b}' rendered identically — variant not resolved",
            );
        }
    }
}

/// Monospace bold must differ from monospace regular.
#[test]
fn mono_bold_renders_distinctly() {
    cosmic_golden::init();
    let mut renderer = HeadlessRenderer::new();
    let regular: cosmic::Element<'_, ()> = cosmic::widget::text::monotext("mono 123")
        .font(cosmic::font::mono())
        .into();
    let bold: cosmic::Element<'_, ()> = cosmic::widget::text::monotext("mono 123")
        .font(cosmic::iced::core::Font {
            weight: font::Weight::Bold,
            ..cosmic::font::mono()
        })
        .into();
    let a = renderer.render(regular, 220, 40);
    let b = renderer.render(bold, 220, 40);
    assert_ne!(a, b, "mono bold rendered identically to mono regular");
}

#[golden_test(220, 40)]
fn text_light() -> cosmic::Element<'static, ()> {
    styled_text(font::Weight::Light, font::Style::Normal)
}

#[golden_test(220, 40)]
fn text_semibold() -> cosmic::Element<'static, ()> {
    styled_text(font::Weight::Semibold, font::Style::Normal)
}

#[golden_test(220, 40)]
fn text_bold() -> cosmic::Element<'static, ()> {
    styled_text(font::Weight::Bold, font::Style::Normal)
}

#[golden_test(220, 40)]
fn text_italic() -> cosmic::Element<'static, ()> {
    styled_text(font::Weight::Normal, font::Style::Italic)
}

#[golden_test(220, 40)]
fn text_bold_italic() -> cosmic::Element<'static, ()> {
    styled_text(font::Weight::Bold, font::Style::Italic)
}

/// `text::title1` uses `font::bold()`; covers the widget-level path.
#[golden_test(320, 60)]
fn text_title1() -> cosmic::Element<'static, ()> {
    cosmic::widget::text::title1("Title One").into()
}

/// `text::heading` uses `font::semibold()`; covers the widget-level path.
#[golden_test(320, 40)]
fn text_heading() -> cosmic::Element<'static, ()> {
    cosmic::widget::text::heading("A Heading").into()
}

#[golden_test(220, 40)]
fn monotext_bold() -> cosmic::Element<'static, ()> {
    cosmic::widget::text::monotext("mono 123")
        .font(cosmic::iced::core::Font {
            weight: font::Weight::Bold,
            ..cosmic::font::mono()
        })
        .into()
}

fn generic_family_text(family: font::Family) -> cosmic::Element<'static, ()> {
    let font = cosmic::iced::core::Font {
        family,
        ..cosmic::font::default()
    };
    cosmic::widget::text("Grumpy wizards 123").font(font).into()
}

/// The three pinned generic families must resolve to three distinct faces.
#[test]
fn generic_families_render_distinctly() {
    cosmic_golden::init();
    let mut renderer = HeadlessRenderer::new();
    let serif = renderer.render(generic_family_text(font::Family::Serif), 220, 40);
    let sans = renderer.render(generic_family_text(font::Family::SansSerif), 220, 40);
    let mono = renderer.render(generic_family_text(font::Family::Monospace), 220, 40);
    assert_ne!(serif, sans, "serif rendered identically to sans-serif");
    assert_ne!(serif, mono, "serif rendered identically to monospace");
    assert_ne!(sans, mono, "sans-serif rendered identically to monospace");
}

#[golden_test(220, 40)]
fn text_serif() -> cosmic::Element<'static, ()> {
    generic_family_text(font::Family::Serif)
}

#[golden_test(220, 40)]
fn text_serif_bold_italic() -> cosmic::Element<'static, ()> {
    let font = cosmic::iced::core::Font {
        family: font::Family::Serif,
        weight: font::Weight::Bold,
        style: font::Style::Italic,
        ..cosmic::font::default()
    };
    cosmic::widget::text("Grumpy wizards 123").font(font).into()
}
