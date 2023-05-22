pub use css_color_parser::Color;
use log::warn;
use simplecss::Declaration;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Copy)]
pub enum FontFamily {
    #[default]
    Regular,
    Bold,
    Mono,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Sides {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Radius {
    pub nw: f32,
    pub ne: f32,
    pub sw: f32,
    pub se: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Align {
    #[default]
    Min,
    Center,
    Max,
}

fn find_and_resolve_property<T>(
    declarations: &[Declaration<'_>],
    property_name: &str,
    variables: &HashMap<String, String>,
) -> Option<T>
where
    T: FromStr,
{
    declarations
        .iter()
        .find(|dec| dec.name == property_name)
        .map(|dec| {
            if dec.value.starts_with("var(") && dec.value.ends_with(')') {
                let last = dec.value.len() - 1;
                let var = dec.value.get(4..last).unwrap();
                if let Some(value) = variables.get(var) {
                    return value.parse();
                }
            }
            dec.value.parse()
        })
        .transpose()
        .unwrap_or_else(|_| {
            warn!("Can't parse `{}` value", property_name);
            None
        })
}

macro_rules! prop_type {
  ($vis:vis struct $name:ident {
    $($field:ident($key:expr) : $type:ty = $value:expr),* $(,)?
  }) => {

    #[derive(Debug, Clone)]
    pub struct ComputedProperties {
      $(
        pub $field: $type
      ),*
    }

    impl Default for ComputedProperties {
      fn default() -> Self {
        Self {
          $(
            $field: $value
          ),*
        }
      }
    }

    impl ComputedProperties {
      pub fn patch_from(&mut self, props: &OptionalProperties) {
        $(
          if let Some(v) = props.$field { self.$field = v; }
        )*
      }
    }


    #[derive(Default)]
    $vis struct OptionalProperties {
      $(
        $field: Option<$type>
      ),*
    }

    pub fn parse_declarations(decs: &[Declaration<'_>], vars: &HashMap<String, String>) -> OptionalProperties {
      for dec in decs {
        if ($( dec.name != $key &&)* true) {
          warn!("Unknown property: {}", dec.name);
        }
      }
      OptionalProperties {
        $(
          $field: find_and_resolve_property(decs, $key, vars),
        )*
      }
    }
  }
}

const INVALID_COLOR: Color = Color {
    r: 255,
    g: 0,
    b: 255,
    a: 1.0,
};

prop_type! {
  pub struct Properties {
    font_size("font-size"): f32 = 4.0,
    font_family("font-family"): FontFamily = FontFamily::default(),
    color("color"): Color = INVALID_COLOR,
    background("background"): Color = INVALID_COLOR,
    italics("italics"): bool = false,
    underline_width("underline-width"): f32 = 0.0,
    underline_color("underline-color"): Color = INVALID_COLOR,
    strikethrough_width("strikethrough-width"): f32 = 0.0,
    strikethrough_color("strikethrough-color"): Color = INVALID_COLOR,
    align("align"): Align = Align::Min,
    cross_align("cross-align"): Align = Align::Min,
    padding("padding"): Sides = Sides::default(),
    margin("margin"): Sides = Sides::default(),
    radius("radius"): Radius = Radius::default(),
    height("height"): f32 = 0.0,
    width("width"): f32 = 0.0,
    min_width("min-width"): f32 = 0.0,
    max_width("max-width"): f32 = f32::INFINITY,
    min_height("min-height"): f32 = 0.0,
    max_height("max-height"): f32 = f32::INFINITY,
    border_width("border-width"): f32 = 0.0,
    border_color("border-color"): Color = INVALID_COLOR,
    stroke_width("stroke-width"): f32 = 0.0,
    stroke_color("stroke-color"): Color = INVALID_COLOR,
    expansion("expansion"): f32 = 0.0,
  }
}
