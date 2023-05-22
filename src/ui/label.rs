use crate::styling::{element::ComputedElement, BoxProperties};
use egui::text::{LayoutJob, TextFormat};
use egui::{
    vec2, Color32, Response, Sense, TextStyle, Ui, Widget, WidgetInfo, WidgetText, WidgetType,
};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct Label {
    text: WidgetText,
    box_properties: BoxProperties,
    sense: Sense,
}

impl Label {
    pub fn new(celt: &ComputedElement, text: &str) -> Self {
        let mut job = LayoutJob::default();
        let text_format: TextFormat = celt.clone().into();
        job.append(text, 0.0, text_format);
        let box_properties: BoxProperties = celt.clone().into();
        Self {
            text: job.into(),
            box_properties,
            sense: Sense::hover(),
        }
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        let wrap = Some(false);
        let wrap_width = f32::INFINITY; // Never wrap
        let style_fallback = TextStyle::Body;
        let text = self.text.into_galley(ui, wrap, wrap_width, style_fallback);

        let text_size = text.size();

        let mut height = if self.box_properties.height != 0.0 {
            self.box_properties.height
        } else {
            text_size.y + self.box_properties.padding.sum().y
        };
        height = height.clamp(
            self.box_properties.min_height,
            self.box_properties.max_height,
        );

        let mut width = if self.box_properties.width != 0.0 {
            self.box_properties.width
        } else {
            text_size.x + self.box_properties.padding.sum().x
        };
        width = width.clamp(self.box_properties.min_width, self.box_properties.max_width);

        let size = vec2(width, height);

        let (rect, response) = ui.allocate_exact_size(size, self.sense);

        // FIXME: I'm sure there's a method to do that better
        let mut padding_less = rect;
        padding_less.min.x += self.box_properties.padding.left;
        padding_less.min.y += self.box_properties.padding.top;
        padding_less.max.x -= self.box_properties.padding.right;
        padding_less.max.y -= self.box_properties.padding.bottom;

        let text_pos = self
            .box_properties
            .align
            .align_size_within_rect(text.size(), padding_less);

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, text.text()));

        if ui.is_rect_visible(rect) {
            ui.scope(|ui| {
                ui.painter().rect(
                    rect.expand(self.box_properties.border.width),
                    self.box_properties.rounding,
                    self.box_properties.background,
                    self.box_properties.border,
                );
                let clip = ui.clip_rect();
                ui.set_clip_rect(padding_less.intersect(clip));
                text.paint_with_fallback_color(ui.painter(), text_pos.min, Color32::RED);
                ui.set_clip_rect(clip);
            });
        }

        response
    }
}
