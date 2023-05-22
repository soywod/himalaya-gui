use egui::{Frame, ScrollArea, Ui};

use crate::css::Rules;
use crate::server::Envelope;
use crate::styling::element as elt;
use crate::ui::label::Label;

pub fn update(ui: &mut Ui, rules: &Rules, envelope: &Envelope, body: Option<&String>) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // FIXME NOW: implement a FullWidthFrame ui widget.
            // Nope. Main frames show justify content.
            // FIXME: this should go beyond styling. And include vertical/horizontal layout
            let headers_elt = elt::vbox().id("body-headers");
            let frame: Frame = headers_elt.compute(rules).into();
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let hbox = elt::hbox()
                            .id("body-headers-from")
                            .parent(headers_elt.clone());
                        ui.horizontal(|ui| {
                            let label_elt = elt::label().classes("label").parent(hbox.clone());
                            let value_elt = elt::label().classes("value").parent(hbox.clone());
                            ui.add(Label::new(&label_elt.compute(rules), "From: "));
                            ui.add(Label::new(&value_elt.compute(rules), &envelope.from.addr));
                        });

                        let hbox = elt::hbox()
                            .id("body-headers-subject")
                            .parent(headers_elt.clone());
                        ui.horizontal(|ui| {
                            let label_elt = elt::label().classes("label").parent(hbox.clone());
                            let value_elt = elt::label().classes("value").parent(hbox.clone());
                            ui.add(Label::new(&label_elt.compute(rules), "Subject: "));
                            ui.add(Label::new(&value_elt.compute(rules), &envelope.subject));
                        });

                        let hbox = elt::hbox()
                            .id("body-headers-date")
                            .parent(headers_elt.clone());
                        ui.horizontal(|ui| {
                            let label_elt = elt::label().classes("label").parent(hbox.clone());
                            let value_elt = elt::label().classes("value").parent(hbox.clone());
                            ui.add(Label::new(&label_elt.compute(rules), "Date: "));
                            let date = envelope.date.to_rfc3339();
                            ui.add(Label::new(&value_elt.compute(rules), &date));
                        });
                    });
                    ui.add_space(ui.available_width());
                });
            });

            let frame: Frame = elt::hbox().id("body-content").compute(rules).into();
            frame.show(ui, |ui| {
                if let Some(body) = body {
                    ui.label(body);
                } else {
                    ui.vertical_centered(|ui| ui.spinner());
                }
            });
        });
}
