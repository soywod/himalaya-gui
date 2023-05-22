use egui::{pos2, Frame, Id, Rect, ScrollArea, Sense, Ui};

use crate::css::Rules;
use crate::server::{Envelope, Flag};
use crate::styling::element::{ComputedElement, Element};
use crate::styling::{element as elt, BoxProperties};
use crate::ui::label::Label;

// FIXME: lot of vecs. could use arrays.
// FIXME: A lot could be computed in `new`, but then the theme live reload would
// not work. We need an invalidation system.
pub fn update(
    ui: &mut Ui,
    rules: &Rules,
    envelopes: &Vec<Envelope>,
    selection: &Option<usize>,
    show_selection: bool,
) -> Option<usize> {
    let mut clicked_row = None;

    let frame: Frame = elt::hbox().id("envelopes-frame").compute(rules).into();

    frame.show(ui, |ui| {
        let row_height = calculate_row_height(rules);
        let available_width = ui.available_width() - ui.style().spacing.scroll_bar_width;
        let cell_widths = calculate_cell_expansion(rules, available_width);

        if show_selection {
            self::show_selection(ui, row_height, selection);
        }

        let clip = ui.available_rect_before_wrap();
        let scroll = ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, envelopes.len(), |ui, row_range| {
                for index in row_range {
                    let envelope = &envelopes[index];

                    let selected = selection
                        .map(|selection| index == selection)
                        .unwrap_or(false);

                    let mut row_elt = create_row_elt();
                    row_elt.toggle_class("selected", selected);
                    envelope.flags.iter().for_each(|flag| {
                        match flag {
                            Flag::Flagged => row_elt.add_class("flagged"),
                            Flag::Answered => row_elt.add_class("answered"),
                            Flag::Deleted => row_elt.add_class("deleted"),
                            Flag::Draft => row_elt.add_class("draft"),
                            Flag::Seen => row_elt.add_class("seen"),
                            Flag::Custom(_) => {}
                        };
                    });

                    if !envelope.flags.contains(&Flag::Seen) {
                        row_elt.add_class("unread");
                    }

                    let mut cells = create_cell_elts();
                    for cell in &mut cells {
                        cell.attach_parent(row_elt.clone());
                    }

                    let props: BoxProperties = row_elt.compute(rules).into();
                    let mut row_bg = ui.available_rect_before_wrap();
                    row_bg.set_height(props.height);

                    let visible_rect = row_bg.intersect(clip);
                    ui.set_clip_rect(visible_rect);

                    ui.painter()
                        .rect(row_bg, props.rounding, props.background, props.border);

                    // Draw labels ========== //

                    let labels = build_labels(envelope);

                    let celts: Vec<ComputedElement> = cells
                        .iter()
                        .zip(&cell_widths)
                        .map(|(elt, width)| {
                            let mut celt = elt.compute(rules);
                            celt.props_mut().width = *width;
                            celt.props_mut().height = row_height;
                            celt
                        })
                        .collect();

                    ui.allocate_ui_at_rect(row_bg, |ui| {
                        ui.horizontal(|ui| {
                            for (label, celt) in labels.iter().zip(celts) {
                                let widget = Label::new(&celt, label).sense(Sense::click());
                                if ui.add(widget).clicked() {
                                    clicked_row = Some(index);
                                }
                            }
                        });
                    });
                }
            });
        let scroll_offset = scroll.state.offset.y;
        ui.data_mut(|x| x.insert_temp(Id::new("scroll_offset"), scroll_offset));
    });

    clicked_row
}

fn calculate_row_height(rules: &Rules) -> f32 {
    // Create dummy element for layout computation
    let box_props: BoxProperties = create_row_elt().compute(rules).into();
    box_props.height
}

fn show_selection(ui: &Ui, row_height: f32, selection: &Option<usize>) {
    if selection.is_some() {
        // FIXME: this should be doable with ScrollArea. But there's no way at the
        // moment.
        let scroll_offset = ui.data_mut(|x| x.get_temp::<f32>(Id::new("scroll_offset")));
        if let Some(scroll_offset) = scroll_offset {
            let sel = selection.unwrap();
            let area_offset = ui.cursor();
            let y = area_offset.top() - scroll_offset
                + sel as f32 * (row_height + ui.spacing().item_spacing.y);
            let rect = Rect::from_two_pos(pos2(0.0, y), pos2(0.0, y + row_height));
            ui.scroll_to_rect(rect, None);
        }
    }
}

fn create_row_elt() -> Element {
    elt::hbox().classes("envelope-row")
}

fn create_cell_elts() -> Vec<Element> {
    vec![
        elt::label().classes("flags-cell"),
        elt::label().classes("sender-cell"),
        elt::label().classes("subject-cell"),
        elt::label().classes("date-cell"),
    ]
}

fn calculate_cell_expansion(rules: &Rules, total_width: f32) -> Vec<f32> {
    // Dummy cells for layout computation
    let cells = create_cell_elts();
    let props: Vec<BoxProperties> = cells
        .iter()
        .map(|cell| cell.compute(rules).into())
        .collect();

    let (non_fexible_width, flexible_count) = props.iter().fold((0.0, 0), |(nfw, fc), celt| {
        if celt.width == 0.0 {
            (nfw, fc + 1)
        } else {
            (nfw + celt.width, fc)
        }
    });

    let flexible_width = (total_width - non_fexible_width) / flexible_count as f32;

    props
        .iter()
        .map(|p| {
            if p.width == 0.0 {
                flexible_width
            } else {
                p.width
            }
        })
        .collect()
}

fn build_label_flag(envelope: &Envelope) -> String {
    let mut label = "".to_owned();
    if !envelope.flags.contains(&Flag::Seen) {
        label.push('');
    }
    for flag in envelope.flags.iter() {
        match flag {
            Flag::Flagged => label.push_str(" "),
            Flag::Answered => label.push_str(" ﬌"),
            Flag::Deleted => label.push_str(" D"),
            Flag::Draft => label.push_str(" d"),
            _ => {}
        };
    }
    label.trim().into()
}

fn build_label_sender(envelope: &Envelope) -> String {
    envelope.from.addr.clone()
}

fn build_label_subject(envelope: &Envelope) -> String {
    envelope
        .subject
        .lines()
        .next()
        .unwrap_or("No subject")
        .to_owned()
}

fn build_label_date(envelope: &Envelope) -> String {
    // FIXME: mirror Vec<Envelope> to its labels counterpart
    envelope.date.to_rfc3339()
}

fn build_labels(envelope: &Envelope) -> Vec<String> {
    vec![
        build_label_flag(envelope),
        build_label_sender(envelope),
        build_label_subject(envelope),
        build_label_date(envelope),
    ]
}
