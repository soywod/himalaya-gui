//! Subsets of CSS properties.
//! Translates CSS properties in stuctures understood by the UI toolkit.

use egui::style::{Margin, Selection, WidgetVisuals};
use egui::{epaint, Align, Align2, Color32, FontId, Frame, Rounding, Stroke, TextFormat};

use crate::css::{
    Align as CSSAlign, Color as CSSColor, FontFamily as CSSFontFamily, Radius, Sides,
};

use super::element::ComputedElement;
use super::fonts;

pub struct TextColors {
    pub fg: Color32,
    pub bg: Color32,
}

pub struct BoxProperties {
    pub align: Align2,
    pub padding: Margin,
    pub rounding: Rounding,
    pub height: f32,
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub background: Color32,
    pub border: Stroke,
}

fn to_color(color: &CSSColor) -> Color32 {
    Color32::from_rgb(color.r, color.g, color.b).linear_multiply(color.a)
}

fn to_align(align: &CSSAlign) -> Align {
    match align {
        CSSAlign::Min => Align::Min,
        CSSAlign::Center => Align::Center,
        CSSAlign::Max => Align::Max,
    }
}

fn to_rounding(radius: &Radius) -> Rounding {
    Rounding {
        nw: radius.nw,
        ne: radius.ne,
        sw: radius.sw,
        se: radius.se,
    }
}

fn to_margin(sides: &Sides) -> Margin {
    Margin {
        top: sides.top,
        left: sides.left,
        bottom: sides.bottom,
        right: sides.right,
    }
}

impl From<ComputedElement> for WidgetVisuals {
    fn from(e: ComputedElement) -> WidgetVisuals {
        WidgetVisuals {
            weak_bg_fill: to_color(&e.0.background),
            bg_fill: to_color(&e.0.background),
            bg_stroke: Stroke {
                color: to_color(&e.0.border_color),
                width: e.0.border_width,
            },
            fg_stroke: Stroke {
                color: to_color(&e.0.stroke_color),
                width: e.0.stroke_width,
            },
            rounding: Rounding {
                nw: e.0.radius.nw,
                ne: e.0.radius.ne,
                sw: e.0.radius.sw,
                se: e.0.radius.se,
            },
            expansion: e.0.expansion,
        }
    }
}

impl From<ComputedElement> for Selection {
    fn from(e: ComputedElement) -> Selection {
        Selection {
            bg_fill: to_color(&e.0.background),
            stroke: Stroke {
                color: to_color(&e.0.stroke_color),
                width: e.0.stroke_width,
            },
        }
    }
}

impl From<ComputedElement> for TextFormat {
    fn from(e: ComputedElement) -> TextFormat {
        TextFormat {
            font_id: {
                let family = match e.0.font_family {
                    CSSFontFamily::Regular => fonts::regular(),
                    CSSFontFamily::Bold => fonts::bold(),
                    CSSFontFamily::Mono => fonts::mono(),
                };
                FontId::new(e.0.font_size, family)
            },
            color: to_color(&e.0.color),
            background: to_color(&e.0.background),
            italics: e.0.italics,
            valign: to_align(&e.0.align),
            strikethrough: Stroke {
                width: e.0.strikethrough_width,
                color: to_color(&e.0.strikethrough_color),
            },
            underline: Stroke {
                width: e.0.underline_width,
                color: to_color(&e.0.underline_color),
            },
        }
    }
}

impl From<ComputedElement> for BoxProperties {
    fn from(e: ComputedElement) -> BoxProperties {
        BoxProperties {
            align: Align2([to_align(&e.0.align), to_align(&e.0.cross_align)]),
            padding: to_margin(&e.0.padding),
            rounding: to_rounding(&e.0.radius),
            min_width: e.0.min_width,
            max_width: e.0.max_width,
            min_height: e.0.min_height,
            max_height: e.0.max_height,
            background: to_color(&e.0.background),
            width: e.0.width,
            height: e.0.height,
            border: Stroke {
                color: to_color(&e.0.border_color),
                width: e.0.border_width,
            },
        }
    }
}

impl From<ComputedElement> for Frame {
    fn from(e: ComputedElement) -> Frame {
        Frame {
            inner_margin: to_margin(&e.0.padding),
            outer_margin: to_margin(&e.0.margin),
            rounding: to_rounding(&e.0.radius),
            shadow: epaint::Shadow::default(),
            fill: to_color(&e.0.background),
            stroke: Stroke {
                color: to_color(&e.0.border_color),
                width: e.0.border_width,
            },
        }
    }
}

impl From<ComputedElement> for TextColors {
    fn from(e: ComputedElement) -> TextColors {
        TextColors {
            fg: to_color(&e.0.color),
            bg: to_color(&e.0.background),
        }
    }
}
