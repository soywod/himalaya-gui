use crate::css::Rules;
use crate::styling::element as elt;
use egui::{CollapsingHeader, Sense, Ui};

use crate::server::{MboxId, Mboxes};
use crate::ui::label::Label;

fn guess_icon(mbox: &str) -> &'static str {
    if mbox.eq_ignore_ascii_case("trash") {
        ""
    } else if mbox.eq_ignore_ascii_case("inbox") {
        ""
    } else if mbox.eq_ignore_ascii_case("sent mail") || mbox.eq_ignore_ascii_case("sent") {
        ""
    } else if mbox.eq_ignore_ascii_case("archive") {
        ""
    } else if mbox.eq_ignore_ascii_case("drafts") {
        ""
    } else if mbox.eq_ignore_ascii_case("spam") {
        ""
    } else {
        ""
    }
}

// FIXME: make sure all updates fonction only take Rules, not Theme
pub fn update(
    ui: &mut Ui,
    rules: &Rules,
    mboxes: &Mboxes,
    selected_mbox: &Option<MboxId>,
) -> Option<MboxId> {
    let mut ret = None;
    for (account, mboxes) in mboxes {
        CollapsingHeader::new(account.trim())
            .default_open(true)
            .show(ui, |ui| {
                for mbox in mboxes {
                    let selected = selected_mbox.as_ref().map_or(false, |selected_mbox| {
                        &selected_mbox.account == account && &selected_mbox.name == mbox
                    });

                    let mut hbox = elt::hbox().classes("folder-listitem");
                    hbox.toggle_class("selected", selected);
                    let icon_elt = elt::label()
                        .classes("icon")
                        .parent(hbox.clone())
                        .compute(rules);
                    let name_elt = elt::label()
                        .classes("name")
                        .parent(hbox.clone())
                        .compute(rules);

                    ui.horizontal(|ui| {
                        let display_name = if mbox.starts_with("[Gmail]/") {
                            mbox.get(8..).unwrap()
                        } else {
                            mbox
                        };
                        let label1 =
                            Label::new(&icon_elt, guess_icon(display_name)).sense(Sense::click());
                        let label2 = Label::new(&name_elt, display_name).sense(Sense::click());
                        let clicked1 = ui.add(label1).clicked();
                        let clicked2 = ui.add(label2).clicked();
                        if !selected && (clicked1 || clicked2) {
                            ret = Some(MboxId {
                                account: account.clone(),
                                name: mbox.clone(),
                            });
                        }
                    });
                }
            });
    }

    ret
}
