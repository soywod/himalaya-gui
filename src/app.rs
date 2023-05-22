use std::collections::HashMap;
use std::sync::mpsc::TryRecvError;

use crate::styling::{element as elt, BoxProperties};
use anyhow::Result;
use egui::{
    vec2, CentralPanel, Context, Frame, Key, Modifiers, Rgba, ScrollArea, SidePanel,
    TopBottomPanel, Ui, Visuals,
};
use log::{error, warn};

use crate::server::{
    self, AccountId, Envelopes, MailId, MboxId, Mboxes, Server, ServerCmd, ServerEvent,
};
use crate::theme::Theme;
use crate::ui;
use crate::ui::label::Label;

pub struct App {
    theme: Theme,
    show_sidebar: bool,
    servers: HashMap<AccountId, Server>,

    mboxes: Mboxes,
    bodies: HashMap<MailId, String>,
    envelopes: Envelopes,
    selected_mbox: Option<MboxId>,
    selected_row: Option<usize>,

    scrolling_necessary: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let ctx = &cc.egui_ctx;

        let theme = Theme::init(cc)?;

        let accounts = server::accounts().expect("Account listing failed");

        let servers: HashMap<AccountId, Server> = accounts
            .iter()
            .map(|account_name| {
                let (to, from) = server::run(ctx.clone(), account_name.to_string());
                to.send(ServerCmd::GetMboxes).unwrap();
                (account_name.to_string(), Server { to, from })
            })
            .collect();

        Ok(Self {
            servers,
            theme,
            show_sidebar: true,
            mboxes: HashMap::new(),
            bodies: HashMap::new(),
            envelopes: Envelopes::default(),
            selected_mbox: None,
            selected_row: None,
            scrolling_necessary: false,
        })
    }

    fn consume_events(&mut self) {
        for (account, server) in &self.servers {
            loop {
                match server.from.try_recv() {
                    Ok(event) => {
                        match event {
                            ServerEvent::Error(e) => error!("Server error: {}", e),
                            ServerEvent::Envelopes((mbox, envelopes)) => {
                                if let Some(selected_mbox) = &self.selected_mbox {
                                    if selected_mbox == &mbox {
                                        self.envelopes = envelopes;
                                    }
                                }
                            }
                            ServerEvent::Mboxes(mboxes) => {
                                self.mboxes.insert(account.to_string(), mboxes);
                            }
                            ServerEvent::Body((id, body)) => {
                                self.bodies.insert(id, body);
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        error!("Server thread died");
                    }
                }
            }
        }
    }

    pub fn select_row(&mut self, row: Option<usize>, scrolling_necessary: bool) {
        if row == self.selected_row {
            // Nothing to do
            return;
        }
        if self.envelopes.is_empty() {
            warn!("Setting row selection without envelopes");
            return;
        }
        self.scrolling_necessary = scrolling_necessary;
        if let Some(mut row) = row {
            if row >= self.envelopes.len() {
                row = self.envelopes.len() - 1;
            }
            self.selected_row = Some(row);
            let id = &self.envelopes[row].id;
            if let Some(mbox) = &self.selected_mbox {
                if !self.bodies.contains_key(id) {
                    let cmd = ServerCmd::GetMessageBody(id.clone(), mbox.name.clone());
                    self.servers
                        .get(&mbox.account)
                        .unwrap()
                        .to
                        .send(cmd)
                        .unwrap();
                }
            } else {
                warn!("Inconsistent state: selected envelope without a selected mbox");
            }
        } else {
            self.selected_row = None;
        }
    }

    pub fn consume_keys(&mut self, ui: &mut Ui) {
        let sel = self.selected_row.unwrap_or(0);

        if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::B)) {
            self.show_sidebar = !self.show_sidebar;
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
            self.select_row(Some(sel + 1), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
            self.select_row(Some(sel.saturating_sub(1)), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::PageUp)) {
            self.select_row(Some(sel.saturating_sub(10)), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::PageDown)) {
            self.select_row(Some(sel + 10), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Home)) {
            self.select_row(Some(0), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::G)) {
            self.select_row(Some(0), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::End)) {
            self.select_row(Some(usize::MAX), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::G)) {
            self.select_row(Some(usize::MAX), true);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.select_row(None, false);
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Y)) {
            ui.scroll_with_delta(vec2(0.0, 20.0));
        }

        if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::E)) {
            ui.scroll_with_delta(vec2(0.0, -20.0));
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        // FIXME:&"XX".into() … can we avoid the &
        let frame: Frame = elt::window().compute(&self.theme.rules()).into();
        let rgba: Rgba = frame.fill.into();
        rgba.to_array()
    }

    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.scrolling_necessary = false;
        self.consume_events();

        // ┌───────┬─────────┐
        // │~~~~~~~│         │
        // │~~~~~~~│         │
        // │~~~~~~~│         │
        // │~~~~~~~├─────────┤
        // │~~~~~~~│         │
        // │~~~~~~~│         │
        // └───────┴─────────┘

        // FIXME: put a lot more in ui::mailboxes
        if self.show_sidebar {
            let frame = elt::panel()
                .id("mailboxespanel")
                .compute(&self.theme.rules())
                .into();
            SidePanel::left("main::left-side-panel")
                .frame(frame)
                .show(ctx, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.mboxes.is_empty() {
                                ui.centered_and_justified(|ui| ui.spinner());
                            } else {
                                // FIXME: return None if no NEW mailbox is selected.
                                // FIXME: that's ugly
                                if let Some(mbox) = ui::mailboxes::update(
                                    ui,
                                    &self.theme.rules(),
                                    &self.mboxes,
                                    &self.selected_mbox,
                                ) {
                                    self.selected_mbox = Some(mbox.clone());
                                    self.scrolling_necessary = false;
                                    self.selected_row = None;
                                    self.envelopes.clear();
                                    self.bodies.clear();
                                    let cmd = ServerCmd::GetAllEnvelopes(mbox.name.clone());
                                    self.servers
                                        .get(&mbox.account)
                                        .unwrap()
                                        .to
                                        .send(cmd)
                                        .unwrap();
                                }
                            }
                        });
                });
        }

        // ┌───────┬─────────┐
        // │       │         │
        // │       │         │
        // │       │         │
        // │       ├─────────┤
        // │       │~~~~~~~~~│
        // │       │~~~~~~~~~│
        // └───────┴─────────┘

        if let Some(row) = self.selected_row {
            let envelope = &self.envelopes[row];

            let mut elt = elt::panel().id("mailbodypanel");
            elt.toggle_class("sidebaropen", self.show_sidebar);
            let computed = elt.compute(&self.theme.rules());
            let frame = computed.clone().into();
            let box_props: BoxProperties = computed.into();
            TopBottomPanel::bottom("my_panel")
                .frame(frame)
                .min_height(box_props.min_height)
                .default_height(box_props.height)
                .resizable(true)
                .show(ctx, |ui| {
                    ui::mailbody::update(
                        ui,
                        &self.theme.rules(),
                        envelope,
                        self.bodies.get(&envelope.id),
                    );
                });
        }

        // ┌───────┬─────────┐
        // │       │~~~~~~~~~│
        // │       │~~~~~~~~~│
        // │       │~~~~~~~~~│
        // │       ├─────────┤
        // │       │         │
        // │       │         │
        // └───────┴─────────┘

        let mut elt = elt::panel().id("mainpanel");
        elt.toggle_class("sidebaropen", self.show_sidebar);
        let frame = elt.compute(&self.theme.rules()).into();
        CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.consume_keys(ui);
            ui.vertical(|ui| {
                let button_clicked = ui::toolbar::update(ui, &self.theme.rules(), &elt);
                if button_clicked {
                    self.show_sidebar = !self.show_sidebar;
                }
                if self.selected_mbox.is_none() {
                    ui.centered_and_justified(|ui| {
                        let celt = elt::label()
                            .id("no-mailbox-label")
                            .compute(&self.theme.rules());
                        let label = Label::new(&celt, "No mailbox selected");
                        ui.add(label);
                    });
                } else if self.envelopes.is_empty() {
                    ui.centered_and_justified(|ui| ui.spinner());
                } else {
                    let clicked_row = ui::envelopes::update(
                        ui,
                        &self.theme.rules(),
                        &self.envelopes,
                        &self.selected_row,
                        self.scrolling_necessary,
                    );
                    if clicked_row.is_some() {
                        self.select_row(clicked_row, false);
                    }
                }
            });
        });
    }
}
