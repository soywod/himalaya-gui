use std::collections::BTreeMap;

use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

pub fn regular() -> FontFamily {
    FontFamily::Name("Regular".into())
}

pub fn mono() -> FontFamily {
    FontFamily::Name("Mono".into())
}
pub fn bold() -> FontFamily {
    FontFamily::Name("Bold".into())
}

pub fn text_styles_for_size(size: f32) -> BTreeMap<TextStyle, FontId> {
    let mut text_styles = BTreeMap::new();

    text_styles.insert(TextStyle::Small, FontId::new(size, regular()));
    text_styles.insert(TextStyle::Body, FontId::new(size, regular()));
    text_styles.insert(TextStyle::Monospace, FontId::new(size, mono()));
    text_styles.insert(TextStyle::Button, FontId::new(size, regular()));
    text_styles.insert(TextStyle::Heading, FontId::new(size, bold()));

    text_styles
}

pub fn register(ctx: &Context) {
    // Register fonts

    let font_data_regular = FontData::from_static(include_bytes!(
        "../../assets/inconsolata-nerd-font/regular.ttf"
    ));
    let font_data_mono = FontData::from_static(include_bytes!(
        "../../assets/inconsolata-nerd-font/mono.ttf"
    ));
    let font_data_bold = FontData::from_static(include_bytes!(
        "../../assets/inconsolata-nerd-font/bold.ttf"
    ));

    let font_name_regular = "inconsolata-nerd-regular".to_owned();
    let font_name_mono = "inconsolata-nerd-mono".to_owned();
    let font_name_bold = "inconsolata-nerd-bodl".to_owned();

    let mut fonts = FontDefinitions::empty();

    fonts
        .families
        .insert(regular(), vec![font_name_regular.clone()]);
    fonts.families.insert(mono(), vec![font_name_mono.clone()]);
    fonts.families.insert(bold(), vec![font_name_bold.clone()]);
    fonts
        .families
        .insert(FontFamily::Proportional, vec![font_name_regular.clone()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec![font_name_mono.clone()]);

    fonts.font_data.insert(font_name_regular, font_data_regular);
    fonts.font_data.insert(font_name_mono, font_data_mono);
    fonts.font_data.insert(font_name_bold, font_data_bold);

    ctx.set_fonts(fonts);
}
