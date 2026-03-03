// ── Barycenter UI ────────────────────────────────────────────────────────────
//
// The iced + iced_layershell application.  Spawns a layer-shell surface on the
// active monitor, requests exclusive keyboard focus, and renders the search
// input + result list using the Cosmic Dawn theme.
//
// v2: SearchEngine reuse, frecency boost, Vim keybindings, graceful exit.
// ─────────────────────────────────────────────────────────────────────────────

pub mod theme;

use crate::config::AppConfig;
use crate::frecency::FrecencyDb;
use crate::runner::{Runner, RunnerResult};
use crate::search::SearchEngine;
use theme::CosmicDawn;

use iced::event::Event;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Color, Element, Length, Task};

const SEARCH_INPUT_ID: &str = "search";

/// Application state.
pub struct Barycenter {
    config: AppConfig,
    theme: CosmicDawn,
    query: String,
    results: Vec<RunnerResult>,
    selected: usize,
    runners: Vec<Box<dyn Runner>>,
    engine: SearchEngine,
    frecency: FrecencyDb,
    /// Set to true when the app should close on the next update cycle.
    should_quit: bool,
}

/// Messages driving the application state machine.
#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Execute,
    SelectPrev,
    SelectNext,
    /// Jump selection to the top of the list.
    SelectTop,
    Dismiss,
    /// Focus the text input (fired once at init).
    FocusInput,
    /// Scroll half-page up.
    HalfPageUp,
    IcedEvent(Event),
}

impl TryInto<iced_layershell::actions::LayershellCustomActions> for Message {
    type Error = Self;
    fn try_into(self) -> Result<iced_layershell::actions::LayershellCustomActions, Self> {
        Err(self)
    }
}

impl Barycenter {
    pub fn new(
        config: AppConfig,
        runners: Vec<Box<dyn Runner>>,
        engine: SearchEngine,
        frecency: FrecencyDb,
    ) -> (Self, Task<Message>) {
        let theme = CosmicDawn::from_config(&config);
        let state = Self {
            config,
            theme,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            runners,
            engine,
            frecency,
            should_quit: false,
        };
        // Auto-focus the search input on launch.
        let focus_task = Task::done(Message::FocusInput);
        (state, focus_task)
    }

    /// Re-query all runners with the current input.
    fn refresh_results(&mut self) {
        self.results.clear();
        let max = self.config.max_results;

        for runner in &self.runners {
            if runner.matches_input(&self.query) {
                let mut hits = runner.query(&self.query, max, &mut self.engine);
                self.results.append(&mut hits);
            }
        }

        // Apply frecency boost.
        if self.config.frecency_enabled {
            for result in &mut self.results {
                let boost = self.frecency.boost(&result.id);
                result.relevance = result.relevance.saturating_add(boost);
            }
        }

        // Sort all results by relevance descending.
        self.results
            .sort_unstable_by(|a, b| b.relevance.cmp(&a.relevance));
        self.results.truncate(max);

        // Reset selection.
        self.selected = 0;
    }
}

// ── iced Application wiring ──────────────────────────────────────────────────

pub fn namespace(_state: &Barycenter) -> String {
    String::from("barycenter")
}

pub fn subscription(_state: &Barycenter) -> iced::Subscription<Message> {
    iced::event::listen().map(Message::IcedEvent)
}

pub fn update(state: &mut Barycenter, message: Message) -> Task<Message> {
    match message {
        Message::FocusInput => {
            return text_input::focus(text_input::Id::new(SEARCH_INPUT_ID));
        }
        Message::QueryChanged(q) => {
            state.query = q;
            state.refresh_results();
        }
        Message::SelectNext => {
            if !state.results.is_empty() {
                state.selected = (state.selected + 1).min(state.results.len() - 1);
            }
        }
        Message::SelectPrev => {
            state.selected = state.selected.saturating_sub(1);
        }
        Message::SelectTop => {
            state.selected = 0;
        }
        Message::HalfPageUp => {
            state.selected = state.selected.saturating_sub(5);
        }
        Message::Execute => {
            if let Some(result) = state.results.get(state.selected) {
                let id = result.id.clone();
                let source = result.source.clone();

                // Record frecency before launch.
                if state.config.frecency_enabled {
                    state.frecency.record_launch(&id);
                }

                // Find the runner that owns this result via slug.
                for runner in &state.runners {
                    if runner.slug() == source {
                        if let Err(e) = runner.execute(&id) {
                            log::error!("Execution failed: {}", e);
                        }
                        break;
                    }
                }

                // Graceful exit — set flag and return a dismiss task.
                state.should_quit = true;
                return Task::done(Message::Dismiss);
            }
        }
        Message::Dismiss => {
            // Graceful exit: close the layer-shell surface.
            // iced_layershell doesn't expose a clean close mechanism,
            // so we use process::exit but only after all state has been
            // properly saved (frecency written in record_launch).
            std::process::exit(0);
        }
        Message::IcedEvent(Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key,
            modifiers,
            ..
        })) => {
            return handle_key(key, modifiers);
        }
        _ => {}
    }
    Task::none()
}

/// Map keyboard events to messages.  Supports arrows, Tab, and Vim keys.
fn handle_key(key: Key, mods: Modifiers) -> Task<Message> {
    match key {
        Key::Named(iced::keyboard::key::Named::Escape) => Task::done(Message::Dismiss),
        Key::Named(iced::keyboard::key::Named::ArrowDown) => Task::done(Message::SelectNext),
        Key::Named(iced::keyboard::key::Named::ArrowUp) => Task::done(Message::SelectPrev),
        Key::Named(iced::keyboard::key::Named::Enter) => Task::done(Message::Execute),
        Key::Named(iced::keyboard::key::Named::Tab) => Task::done(Message::SelectNext),
        // ── Vim-style bindings (Ctrl held) ───────────────────────────────
        Key::Character(ref c) if mods.control() => match c.as_str() {
            "j" | "n" => Task::done(Message::SelectNext),
            "k" | "p" => Task::done(Message::SelectPrev),
            "u" => Task::done(Message::HalfPageUp),
            "g" => Task::done(Message::SelectTop),
            _ => Task::none(),
        },
        _ => Task::none(),
    }
}

pub fn view(state: &Barycenter) -> Element<'_, Message> {
    let t = &state.theme;
    let cfg = &state.config;

    // ── Search input ─────────────────────────────────────────────────────
    let input = {
        let theme_clone = t.clone();
        text_input("Type to search...", &state.query)
            .on_input(Message::QueryChanged)
            .size(cfg.font_size)
            .padding(14)
            .style(move |_theme, _status| theme::search_input(&theme_clone))
            .id(text_input::Id::new(SEARCH_INPUT_ID))
    };

    // ── Results list ─────────────────────────────────────────────────────
    let result_rows: Vec<Element<Message>> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == state.selected;

            let title = text(&result.title).size(cfg.font_size * 0.82).color(t.text);

            let desc = text(&result.description)
                .size(cfg.font_size * 0.6)
                .color(Color::from_rgba(t.text.r, t.text.g, t.text.b, 0.55));

            let badge_text = match result.source.as_str() {
                "calc" => "CALC",
                "sys" => "SYS",
                "shell" => "SHELL",
                _ => "",
            };

            let content: Element<Message> = if badge_text.is_empty() {
                column![title, desc]
                    .spacing(2)
                    .padding(10)
                    .width(Length::Fill)
                    .into()
            } else {
                let badge = text(badge_text).size(cfg.font_size * 0.5).color(t.accent);
                row![
                    column![title, desc].spacing(2).width(Length::Fill),
                    container(badge).padding([4, 8]).align_y(Alignment::Center),
                ]
                .padding(10)
                .spacing(8)
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .into()
            };

            let style = if is_selected {
                theme::result_row_selected(t)
            } else {
                theme::result_row(t)
            };

            container(content)
                .style(move |_theme| style)
                .width(Length::Fill)
                .into()
        })
        .collect();

    let results_column = Column::with_children(result_rows)
        .spacing(4)
        .width(Length::Fill);

    let results_scroll = scrollable(results_column)
        .height(Length::Fill)
        .style(theme::results_scrollable);

    // ── Assemble layout ──────────────────────────────────────────────────
    let body = column![input, results_scroll]
        .spacing(12)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill);

    let style = theme::window_container(t);
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| style)
        .into()
}

pub fn style(_state: &Barycenter, _theme: &iced::Theme) -> iced_layershell::Appearance {
    iced_layershell::Appearance {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    }
}
