mod client;
mod platform;
mod settings;

use anyhow::Result;
use client::DaemonClient;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::{
    AnyElement, App, AppContext as _, Application, Bounds, Context, FocusHandle, Focusable, Global,
    KeyDownEvent, MouseButton, MouseDownEvent, Render, Rgba, ScrollHandle, Subscription, Window,
    WindowBounds, WindowHandle, WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use palette_protocol::{
    CatalogItem, InsertionPosition, ItemKind, ItemSource, LiveContext, RequestKind, ResponseData,
    ServiceStatus,
};
use settings::{ContentGroup, PaletteSettings};
use std::time::Duration;

const WINDOW_WIDTH: f32 = 760.0;
const WINDOW_HEIGHT: f32 = 520.0;
const RESULT_LIMIT: usize = 200;

struct HotkeyRegistration {
    _manager: GlobalHotKeyManager,
    id: u32,
    window: Option<WindowHandle<CommandPalette>>,
}

impl Global for HotkeyRegistration {}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Activity {
    Idle,
    Searching,
    Running,
    Scanning,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Search,
    Settings,
    ItemActions,
}

struct CommandPalette {
    client: Option<DaemonClient>,
    query: String,
    results: Vec<CatalogItem>,
    selected: usize,
    placement: InsertionPosition,
    context: Option<LiveContext>,
    status: Option<ServiceStatus>,
    notice: String,
    activity: Activity,
    generation: u64,
    active: bool,
    mode: ViewMode,
    settings: PaletteSettings,
    settings_selected: usize,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl CommandPalette {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if this.active && !window.is_window_active() {
                this.close(window, cx);
            }
        });
        let mut palette = Self {
            client: DaemonClient::discover().ok(),
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            placement: InsertionPosition::AfterSelected,
            context: None,
            status: None,
            notice: "Type to search Live's Browser".into(),
            activity: Activity::Idle,
            generation: 0,
            active: false,
            mode: ViewMode::Search,
            settings: PaletteSettings::load(),
            settings_selected: 0,
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            _subscriptions: vec![activation],
        };
        cx.activate(true);
        window.activate_window();
        window.focus(&palette.focus_handle);
        palette.active = true;
        palette.refresh_context(cx);
        palette.search(cx);
        cx.notify();
        palette
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        cx.update_global::<HotkeyRegistration, _>(|registration, _| {
            registration.window = None;
        });
        window.remove_window();
        cx.hide();
    }

    fn refresh_context(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            self.notice = "Palette service is not installed or configured".into();
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let status = client.request(RequestKind::Status)?;
                    let context = client.request(RequestKind::GetContext)?;
                    Ok::<_, anyhow::Error>((status, context))
                })
                .await;
            this.update(cx, |this, cx| {
                match result {
                    Ok((ResponseData::Status(status), ResponseData::Context(context))) => {
                        this.status = Some(status);
                        this.context = Some(context);
                    }
                    Ok(_) => this.notice = "Unexpected response from palette service".into(),
                    Err(error) => this.notice = short_error(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn search(&mut self, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            self.notice = "Palette service is not installed or configured".into();
            return;
        };
        self.generation += 1;
        let generation = self.generation;
        let query = self.query.clone();
        self.activity = Activity::Searching;
        cx.spawn(async move |this, cx| {
            let response = cx
                .background_executor()
                .spawn(async move {
                    client.request(RequestKind::Search {
                        query,
                        limit: RESULT_LIMIT,
                    })
                })
                .await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.activity = Activity::Idle;
                match response {
                    Ok(ResponseData::SearchResults(mut items)) => {
                        items.retain(|item| {
                            this.settings
                                .visibility
                                .is_visible(ContentGroup::for_kind(&item.kind))
                        });
                        items.sort_by_key(|item| ContentGroup::for_kind(&item.kind).order());
                        this.results = items;
                        this.selected = 0;
                        this.scroll_handle = ScrollHandle::new();
                        this.notice = if this.results.is_empty() {
                            "No matches — press ⌘R to refresh the Browser index".into()
                        } else {
                            format!("{} matches", this.results.len())
                        };
                    }
                    Ok(_) => this.notice = "Unexpected search response".into(),
                    Err(error) => this.notice = short_error(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn execute_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.results.get(self.selected).cloned() else {
            return;
        };
        let Some(client) = self.client.clone() else {
            self.notice = "Palette service is not available".into();
            return;
        };
        let placement = self.placement.clone();
        let label = item.name.clone();
        let request = match item.kind {
            ItemKind::Workflow => RequestKind::RunWorkflow {
                workflow_id: item
                    .id
                    .strip_prefix("workflow:")
                    .unwrap_or(&item.id)
                    .to_string(),
            },
            _ => RequestKind::LoadItem {
                item_id: item.id,
                browser_path: item.browser_path,
                position: placement,
            },
        };
        self.activity = Activity::Running;
        self.notice = format!("Adding {label}…");
        cx.spawn_in(window, async move |this, cx| {
            let response = cx
                .background_executor()
                .spawn(async move { client.request(request) })
                .await;
            let succeeded = matches!(
                &response,
                Ok(ResponseData::Ack) | Ok(ResponseData::ActionResults(_))
            );
            if succeeded {
                cx.update(|window, cx| {
                    cx.update_global::<HotkeyRegistration, _>(|registration, _| {
                        registration.window = None;
                    });
                    window.remove_window();
                    cx.hide();
                })
                .ok();
                return;
            }
            this.update(cx, |this, cx| {
                this.activity = Activity::Idle;
                match response {
                    Ok(_) => this.notice = format!("{label} completed"),
                    Err(error) => this.notice = short_error(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.results.is_empty() {
            return;
        }
        self.selected =
            (self.selected as isize + delta).rem_euclid(self.results.len() as isize) as usize;
        self.scroll_handle
            .scroll_to_item(self.result_child_index(self.selected));
        cx.notify();
    }

    fn result_child_index(&self, result_index: usize) -> usize {
        let mut headers = 0;
        let mut prior = None;
        for item in self.results.iter().take(result_index + 1) {
            let group = ContentGroup::for_kind(&item.kind);
            if prior != Some(group) {
                headers += 1;
                prior = Some(group);
            }
        }
        result_index + headers
    }

    fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.mode = ViewMode::Settings;
        self.settings_selected = 0;
        self.notice = "Choose which content groups appear in search".into();
        cx.notify();
    }

    fn open_item_actions(&mut self, cx: &mut Context<Self>) {
        if self.results.get(self.selected).is_some() {
            self.mode = ViewMode::ItemActions;
            cx.notify();
        }
    }

    fn toggle_setting(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(group) = ContentGroup::SETTINGS.get(index).copied() else {
            return;
        };
        self.settings.visibility.toggle(group);
        match self.settings.save() {
            Ok(()) => self.notice = format!("{} preference saved", group.label()),
            Err(error) => self.notice = format!("Could not save settings: {error}"),
        }
        self.search(cx);
        cx.notify();
    }

    fn toggle_selected_favorite(&mut self, cx: &mut Context<Self>) {
        let Some(item) = self.results.get(self.selected).cloned() else {
            return;
        };
        let Some(client) = self.client.clone() else {
            self.notice = "Palette service is not available".into();
            return;
        };
        let favorite = !item.favorite;
        let item_id = item.id;
        let label = item.name;
        self.activity = Activity::Running;
        self.notice = if favorite {
            format!("Adding {label} to favorites…")
        } else {
            format!("Removing {label} from favorites…")
        };
        cx.spawn(async move |this, cx| {
            let response = cx
                .background_executor()
                .spawn(
                    async move { client.request(RequestKind::SetFavorite { item_id, favorite }) },
                )
                .await;
            this.update(cx, |this, cx| {
                this.activity = Activity::Idle;
                match response {
                    Ok(ResponseData::Ack) => {
                        this.mode = ViewMode::Search;
                        this.notice = if favorite {
                            format!("{label} added to favorites")
                        } else {
                            format!("{label} removed from favorites")
                        };
                        this.search(cx);
                    }
                    Ok(_) => this.notice = "Unexpected favorite response".into(),
                    Err(error) => this.notice = short_error(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn scan_browser(&mut self, cx: &mut Context<Self>) {
        if self.activity == Activity::Scanning {
            return;
        }
        let Some(client) = self.client.clone() else {
            self.notice = "Palette service is not available".into();
            return;
        };
        self.activity = Activity::Scanning;
        self.notice = "Refreshing the complete Live Browser index…".into();
        cx.spawn(async move |this, cx| {
            let response = cx
                .background_executor()
                .spawn(async move {
                    client.request(RequestKind::ScanCatalog {
                        max_items: 250_000,
                        max_depth: 64,
                    })
                })
                .await;
            this.update(cx, |this, cx| {
                this.activity = Activity::Idle;
                match response {
                    Ok(ResponseData::CatalogScan(summary)) => {
                        this.notice =
                            format!("Browser refreshed: {} searchable items", summary.scanned);
                        this.search(cx);
                    }
                    Ok(ResponseData::Catalog(items)) => {
                        this.notice =
                            format!("Browser refreshed: {} searchable items", items.len());
                        this.search(cx);
                    }
                    Ok(_) => this.notice = "Unexpected Browser refresh response".into(),
                    Err(error) => this.notice = short_error(error),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn cycle_placement(&mut self, reverse: bool, cx: &mut Context<Self>) {
        const POSITIONS: [InsertionPosition; 4] = [
            InsertionPosition::Beginning,
            InsertionPosition::BeforeSelected,
            InsertionPosition::AfterSelected,
            InsertionPosition::End,
        ];
        let current = POSITIONS
            .iter()
            .position(|position| *position == self.placement)
            .unwrap_or(2);
        let next = if reverse {
            (current + POSITIONS.len() - 1) % POSITIONS.len()
        } else {
            (current + 1) % POSITIONS.len()
        };
        self.placement = POSITIONS[next].clone();
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if modifiers.platform && key == "," {
            if self.mode == ViewMode::Settings {
                self.mode = ViewMode::Search;
                self.search(cx);
            } else {
                self.open_settings(cx);
            }
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "k" && self.mode == ViewMode::Search {
            self.open_item_actions(cx);
            cx.stop_propagation();
            return;
        }
        if self.mode == ViewMode::Settings {
            match key {
                "escape" => {
                    self.mode = ViewMode::Search;
                    cx.notify();
                }
                "down" => {
                    self.settings_selected =
                        (self.settings_selected + 1) % ContentGroup::SETTINGS.len();
                    cx.notify();
                }
                "up" => {
                    self.settings_selected =
                        (self.settings_selected + ContentGroup::SETTINGS.len() - 1)
                            % ContentGroup::SETTINGS.len();
                    cx.notify();
                }
                "enter" | "space" => self.toggle_setting(self.settings_selected, cx),
                _ => {}
            }
            cx.stop_propagation();
            return;
        }
        if self.mode == ViewMode::ItemActions {
            match key {
                "escape" => {
                    self.mode = ViewMode::Search;
                    cx.notify();
                }
                "enter" | "space" => self.toggle_selected_favorite(cx),
                _ => {}
            }
            cx.stop_propagation();
            return;
        }
        match key {
            "escape" => self.close(window, cx),
            "down" => self.move_selection(1, cx),
            "up" => self.move_selection(-1, cx),
            "enter" => self.execute_selected(window, cx),
            "tab" => self.cycle_placement(modifiers.shift, cx),
            "backspace" if !modifiers.platform && !modifiers.control => {
                self.query.pop();
                self.search(cx);
                cx.notify();
            }
            "r" if modifiers.platform => self.scan_browser(cx),
            "v" if modifiers.platform => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.query.push_str(&text.replace(['\r', '\n'], " "));
                    self.search(cx);
                }
            }
            _ if !modifiers.platform && !modifiers.control => {
                if let Some(text) = event.keystroke.key_char.as_deref()
                    && !text.chars().any(char::is_control)
                {
                    self.query.push_str(text);
                    self.search(cx);
                    cx.notify();
                }
            }
            _ => {}
        }
        cx.stop_propagation();
    }

    fn pick(
        &mut self,
        index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected = index;
        self.execute_selected(window, cx);
    }

    fn track_line(&self) -> String {
        let Some(context) = &self.context else {
            return "Waiting for Ableton Live…".into();
        };
        match &context.selected_device_name {
            Some(device) => format!("{}  ·  selected: {}", context.track_name, device),
            None => format!("{}  ·  no device selected", context.track_name),
        }
    }

    fn render_results(
        &mut self,
        surface: Rgba,
        raised: Rgba,
        text: Rgba,
        muted: Rgba,
        accent: Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut list = div()
            .id("results-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle);
        if self.results.is_empty() {
            return list
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(muted)
                        .child(if self.activity == Activity::Searching {
                            "Searching…"
                        } else {
                            "No results"
                        }),
                )
                .into_any_element();
        }

        let mut prior_group = None;
        for (index, item) in self.results.iter().enumerate() {
            let group = ContentGroup::for_kind(&item.kind);
            if prior_group != Some(group) {
                prior_group = Some(group);
                list = list.child(
                    div()
                        .h(px(22.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .px(px(12.))
                        .bg(raised)
                        .text_color(muted)
                        .text_size(px(10.))
                        .child(group.label().to_uppercase()),
                );
            }
            let active = index == self.selected;
            let kind = kind_label(&item.kind);
            let source = source_label(&item.source);
            let path = if item.browser_path.len() > 1 {
                item.browser_path[..item.browser_path.len() - 1].join("  ›  ")
            } else {
                item.categories.join("  ›  ")
            };
            let mut chips = Vec::new();
            if item.pinned {
                chips.push("PINNED");
            }
            if item.favorite {
                chips.push("FAVORITE");
            }
            let detail = if chips.is_empty() {
                format!("{kind} · {source}")
            } else {
                format!("{} · {kind}", chips.join(" · "))
            };
            list = list.child(
                div()
                    .id(("result", index))
                    .flex()
                    .items_center()
                    .h(px(38.))
                    .flex_none()
                    .px(px(12.))
                    .gap(px(12.))
                    .cursor_pointer()
                    .bg(if active { accent } else { surface })
                    .text_color(if active { rgb(0x080808) } else { text })
                    .hover(move |style| style.bg(if active { accent } else { raised }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event, window, cx| {
                            this.pick(index, event, window, cx);
                        }),
                    )
                    .child(
                        div()
                            .w(px(104.))
                            .flex_none()
                            .text_size(px(10.))
                            .child(detail),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(item.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(if active { rgb(0x242424) } else { muted })
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(path),
                            ),
                    ),
            );
        }
        list.into_any_element()
    }

    fn render_settings(
        &mut self,
        surface: Rgba,
        raised: Rgba,
        text: Rgba,
        muted: Rgba,
        accent: Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut list = div()
            .id("settings-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        for (index, group) in ContentGroup::SETTINGS.iter().copied().enumerate() {
            let active = index == self.settings_selected;
            let enabled = self.settings.visibility.is_visible(group);
            list = list.child(
                div()
                    .id(("setting", index))
                    .h(px(40.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(14.))
                    .cursor_pointer()
                    .bg(if active { accent } else { surface })
                    .text_color(if active { rgb(0x080808) } else { text })
                    .hover(move |style| style.bg(if active { accent } else { raised }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.settings_selected = index;
                            this.toggle_setting(index, cx);
                        }),
                    )
                    .child(div().flex_1().child(group.label()))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(if active { rgb(0x242424) } else { muted })
                            .child(if enabled { "SHOWN" } else { "HIDDEN" }),
                    ),
            );
        }
        list.into_any_element()
    }

    fn render_item_actions(
        &mut self,
        surface: Rgba,
        text: Rgba,
        muted: Rgba,
        accent: Rgba,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (label, detail) = self
            .results
            .get(self.selected)
            .map(|item| {
                if item.favorite {
                    ("Remove from favorites", item.name.clone())
                } else {
                    ("Add to favorites", item.name.clone())
                }
            })
            .unwrap_or(("Add to favorites", String::new()));
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .bg(surface)
            .child(
                div()
                    .id("favorite-action")
                    .h(px(46.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(14.))
                    .gap(px(12.))
                    .cursor_pointer()
                    .bg(accent)
                    .text_color(rgb(0x080808))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.toggle_selected_favorite(cx)),
                    )
                    .child(div().flex_1().child(label))
                    .child(div().text_size(px(11.)).text_color(muted).child(detail)),
            )
            .child(
                div()
                    .p(px(14.))
                    .text_size(px(11.))
                    .text_color(text)
                    .child("More item actions can be added here without changing search."),
            )
            .into_any_element()
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = rgb(0x444444);
        let surface = rgb(0x171717);
        let raised = rgb(0x232323);
        let text = rgb(0xf2f2f2);
        let muted = rgb(0x999999);
        let accent = rgb(0xff7a00);
        let title = match self.mode {
            ViewMode::Search if self.query.is_empty() => {
                "Search devices, plug-ins, presets, racks, samples, commands…".to_string()
            }
            ViewMode::Search => format!("{}▎", self.query),
            ViewMode::Settings => "Search content settings".to_string(),
            ViewMode::ItemActions => self
                .results
                .get(self.selected)
                .map(|item| format!("Actions for {}", item.name))
                .unwrap_or_else(|| "Item actions".into()),
        };
        let body = match self.mode {
            ViewMode::Search => self.render_results(surface, raised, text, muted, accent, cx),
            ViewMode::Settings => self.render_settings(surface, raised, text, muted, accent, cx),
            ViewMode::ItemActions => self.render_item_actions(surface, text, muted, accent, cx),
        };
        let help = match self.mode {
            ViewMode::Search => "↑↓ Navigate   ↵ Add   ⌘K Actions   ⌘, Settings   Tab Position",
            ViewMode::Settings => "↑↓ Navigate   Space Toggle   Esc Back",
            ViewMode::ItemActions => "↵ Apply   Esc Back",
        };

        div()
            .id("command-palette-root")
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(surface)
            .text_color(text)
            .font_family("Helvetica Neue")
            .border(px(1.))
            .border_color(border)
            .rounded(px(10.))
            .shadow_lg()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(72.))
                    .flex_none()
                    .px(px(18.))
                    .gap(px(12.))
                    .border_b(px(1.))
                    .border_color(border)
                    .child(div().text_size(px(24.)).text_color(accent).child("⌕"))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(18.))
                            .text_color(if self.query.is_empty() { muted } else { text })
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(title),
                    )
                    .child(
                        div()
                            .px(px(7.))
                            .py(px(4.))
                            .rounded(px(4.))
                            .bg(raised)
                            .text_color(muted)
                            .text_size(px(10.))
                            .child("ESC"),
                    ),
            )
            .child(body)
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(58.))
                    .flex_none()
                    .px(px(14.))
                    .gap(px(12.))
                    .border_t(px(1.))
                    .border_color(border)
                    .bg(raised)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(self.track_line()),
                            )
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(muted)
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(self.notice.clone()),
                            ),
                    )
                    .child(div().text_size(px(10.)).text_color(muted).child(help))
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(5.))
                            .rounded(px(4.))
                            .bg(accent)
                            .text_color(rgb(0x080808))
                            .text_size(px(10.))
                            .child(placement_label(&self.placement)),
                    ),
            )
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn open_palette_window(cx: &mut App) -> Result<WindowHandle<CommandPalette>> {
    let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            kind: WindowKind::PopUp,
            is_movable: false,
            focus: true,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| CommandPalette::new(window, cx)),
    )
}

fn install_hotkey(cx: &mut App) -> Result<()> {
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyJ);
    manager.register(hotkey)?;
    let id = hotkey.id();
    let receiver = GlobalHotKeyEvent::receiver().clone();
    cx.set_global(HotkeyRegistration {
        _manager: manager,
        id,
        window: None,
    });
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(35))
                .await;
            while let Ok(event) = receiver.try_recv() {
                let is_ours = cx
                    .read_global::<HotkeyRegistration, _>(|registration, _| {
                        registration.id == event.id
                    })
                    .unwrap_or(false);
                if is_ours && event.state == global_hotkey::HotKeyState::Released {
                    let existing = cx
                        .read_global::<HotkeyRegistration, _>(|registration, _| registration.window)
                        .ok()
                        .flatten();
                    if let Some(window) = existing
                        && window
                            .update(cx, |palette, window, cx| palette.close(window, cx))
                            .is_ok()
                    {
                        continue;
                    }
                    if platform::ableton_is_frontmost() {
                        cx.update(|cx| {
                            let window = open_palette_window(cx)?;
                            cx.update_global::<HotkeyRegistration, _>(|registration, _| {
                                registration.window = Some(window);
                            });
                            Ok::<_, anyhow::Error>(())
                        })
                        .ok();
                    }
                }
            }
        }
    })
    .detach();
    Ok(())
}

fn main() {
    Application::new().run(|cx: &mut App| {
        install_hotkey(cx).unwrap_or_else(|error| panic!("failed to register Command-J: {error}"));
        cx.hide();
    });
}

fn placement_label(position: &InsertionPosition) -> &'static str {
    match position {
        InsertionPosition::Beginning => "Beginning",
        InsertionPosition::BeforeSelected => "Before selected",
        InsertionPosition::AfterSelected => "After selected",
        InsertionPosition::End => "End",
    }
}

fn kind_label(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::NativeDevice => "DEVICE",
        ItemKind::Plugin => "PLUG-IN",
        ItemKind::Preset => "PRESET",
        ItemKind::Rack => "RACK",
        ItemKind::MaxDevice => "MAX",
        ItemKind::Sample => "SAMPLE",
        ItemKind::Loop => "LOOP",
        ItemKind::Command => "COMMAND",
        ItemKind::Workflow => "WORKFLOW",
        ItemKind::Unknown => "ITEM",
    }
}

fn source_label(source: &ItemSource) -> &'static str {
    match source {
        ItemSource::LiveBrowser => "BROWSER",
        ItemSource::LiveDatabase => "LIVE INDEX",
        ItemSource::OfficialExtension => "EXTENSION",
        ItemSource::User => "USER",
        ItemSource::BuiltIn => "BUILT-IN",
    }
}

fn short_error(error: anyhow::Error) -> String {
    let message = error.to_string();
    if message.is_empty() {
        "Command failed".into()
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_labels_are_human_readable() {
        assert_eq!(
            placement_label(&InsertionPosition::AfterSelected),
            "After selected"
        );
    }

    #[test]
    fn every_catalog_kind_has_a_compact_label() {
        assert_eq!(kind_label(&ItemKind::Sample), "SAMPLE");
        assert_eq!(kind_label(&ItemKind::Plugin), "PLUG-IN");
    }
}
