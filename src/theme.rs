use std::path::Path;
use std::sync::Arc;

use crate::{
    css::{parse_css, Rules},
    styling::{element as elt, fonts, TextColors},
};
use anyhow::Result;
use egui::style::{Interaction, Margin, Spacing, Style, Visuals, Widgets};
use egui::{epaint, vec2, Context, Frame};
use notify::{
    event::{DataChange, EventKind, ModifyKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};

// FIXME:Make it so that we don't need to hold a reference to watchers
// FIXME: and only carry the rules around.
pub struct Theme {
    rules: Arc<RwLock<Rules>>,
    _watcher: RecommendedWatcher,
}

const CSS_PATH: &str = "./assets/theme.css";

fn get_system_classes() -> [&'static str; 2] {
    match dark_light::detect() {
        dark_light::Mode::Dark => [std::env::consts::OS, "dark"],
        dark_light::Mode::Light | dark_light::Mode::Default => [std::env::consts::OS, "light"],
    }
}

impl Theme {
    pub fn init(cc: &eframe::CreationContext<'_>) -> Result<Theme> {
        let ctx = &cc.egui_ctx;

        let dark = matches!(dark_light::detect(), dark_light::Mode::Dark);

        let rules = parse_css(CSS_PATH, &get_system_classes())?;
        let rules = Arc::new(RwLock::new(rules));

        let inner_rules = rules.clone();
        let inner_ctx = cc.egui_ctx.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(e) => {
                        if matches!(
                            e.kind,
                            EventKind::Modify(ModifyKind::Data(DataChange::Content))
                        ) {
                            let rules = parse_css(CSS_PATH, &get_system_classes()).unwrap(); // FIXME
                            let mut inner_rules = inner_rules.write();
                            *inner_rules = rules;
                            update_native_style(&inner_ctx, &inner_rules, dark);
                            inner_ctx.request_repaint();
                        }
                    }
                    Err(e) => {
                        println!("watch error: {:?}", e);
                    }
                }
            })?;
        watcher.watch(Path::new(CSS_PATH), RecursiveMode::NonRecursive)?;

        let theme = Theme {
            _watcher: watcher,
            rules,
        };

        fonts::register(ctx);

        update_native_style(ctx, &theme.rules.read(), dark);

        Ok(theme)
    }

    pub fn rules(&self) -> MappedRwLockReadGuard<'_, Rules> {
        RwLockReadGuard::map(self.rules.read(), |r| r)
    }
}

fn update_native_style(ctx: &Context, rules: &Rules, dark: bool) {
    let native_celt = elt::native().compute(rules);

    let hyperlink_props: TextColors = elt::native().classes("hyperlink").compute(rules).into();
    let faint_props: TextColors = elt::native().classes("faint").compute(rules).into();
    let extreme_props: TextColors = elt::native().classes("extreme").compute(rules).into();
    let code_props: TextColors = elt::native().classes("code").compute(rules).into();
    let warn_props: TextColors = elt::native().classes("warn").compute(rules).into();
    let error_props: TextColors = elt::native().classes("error").compute(rules).into();
    let window_props: Frame = elt::window().compute(rules).into();

    let text_styles = fonts::text_styles_for_size(native_celt.props().font_size);

    ctx.set_style(Style {
        text_styles,
        override_font_id: None,
        override_text_style: None,
        wrap: None,
        // FIXME: not stylable yet
        spacing: Spacing {
            item_spacing: vec2(0.0, 0.0),
            window_margin: Margin::same(6.0),
            button_padding: vec2(4.0, 1.0),
            indent: 18.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
            interact_size: vec2(40.0, 18.0),
            slider_width: 100.0,
            text_edit_width: 280.0,
            icon_width: 14.0,
            icon_width_inner: 8.0,
            icon_spacing: 4.0,
            tooltip_width: 600.0,
            combo_height: 200.0,
            scroll_bar_width: 4.0, // FIXME: need to be themable
            indent_ends_with_horizontal_line: false,
            menu_margin: Margin::same(1.0),
            ..Default::default()
        },
        interaction: Interaction {
            resize_grab_radius_side: 5.0,
            resize_grab_radius_corner: 10.0,
            show_tooltips_only_when_still: false,
        },
        visuals: Visuals {
            dark_mode: dark,
            override_text_color: None,
            widgets: Widgets {
                noninteractive: elt::native()
                    .classes("non-interactive")
                    .compute(rules)
                    .into(),
                open: elt::native().classes("open").compute(rules).into(),
                inactive: elt::native().compute(rules).into(),
                active: elt::native().active(true).compute(rules).into(),
                hovered: elt::native().hover(true).compute(rules).into(),
            },
            selection: elt::native().classes("selection").compute(rules).into(),

            window_shadow: epaint::Shadow::big_dark(),
            popup_shadow: epaint::Shadow::small_dark(),
            resize_corner_size: 12.0,
            text_cursor_width: 2.0,
            text_cursor_preview: false,
            clip_rect_margin: 0.0,
            button_frame: true,
            collapsing_header_frame: false,

            hyperlink_color: hyperlink_props.fg,
            faint_bg_color: faint_props.bg,
            extreme_bg_color: extreme_props.bg,
            code_bg_color: code_props.bg,
            warn_fg_color: warn_props.fg,
            error_fg_color: error_props.fg,
            window_rounding: window_props.rounding,
            ..Default::default()
        },
        animation_time: 1.0 / 12.0,
        debug: Default::default(),
        explanation_tooltips: false,
        ..Default::default()
    });
}
