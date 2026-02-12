// KlinScore - Clinical Score Calculator
// main.rs

mod config;
mod scores;
mod settings;
mod ui;

use config::Specialty;
use scores::{calculate_score, load_all_scores, CalculationResult, ScoreLibrary};
use settings::{AppTheme, Settings};
use ui::{InputMessage, Language, ScoreInputState};

use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text},
    Alignment, Element, Length, Task,
};

fn main() -> iced::Result {
    iced::application("KlinScore", KlinScore::update, KlinScore::view)
        .theme(KlinScore::theme)
        .window_size((1000.0, 700.0))
        .run_with(KlinScore::new)
}

// Application State
#[derive(Debug, Clone)]
enum AppState {
    Loading,
    Welcome,
    SpecialtySelection,
    ScoreSelection {
        specialty: Specialty,
    },
    ScoreCalculation {
        specialty: Specialty,
        score_id: String,
        input_state: ScoreInputState,
        result: Option<CalculationResult>,
        error: Option<String>,
    },
    Settings,
    Error(String),
}

// Main Application
struct KlinScore {
    state: AppState,
    language: Language,
    score_library: Option<ScoreLibrary>,
    settings: Settings,
}

// Messages (user interactions)
#[derive(Debug, Clone)]
enum Message {
    LanguageToggled,
    ScoresLoaded(Result<ScoreLibrary, String>),
    SpecialtySelected(Specialty),
    ScoreSelected(String),
    Input(InputMessage),
    BackToWelcome,
    BackToSpecialtySelection,
    BackToScoreSelection,
    OpenSettings,
    CloseSettings,
    ThemeChanged(AppTheme),
}

impl KlinScore {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            state: AppState::Loading,
            language: Language::German,
            score_library: None,
            settings: Settings::new(),
        };

        // Load scores asynchronously
        let task = Task::perform(
            async {
                match load_all_scores("scores/") {
                    Ok(library) => Ok(library),
                    Err(e) => Err(format!("Failed to load scores: {}", e)),
                }
            },
            Message::ScoresLoaded,
        );

        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LanguageToggled => {
                self.language = match self.language {
                    Language::German => Language::English,
                    Language::English => Language::German,
                };
            }
            Message::ScoresLoaded(result) => {
                match result {
                    Ok(library) => {
                        self.score_library = Some(library);
                        self.state = AppState::Welcome;
                    }
                    Err(e) => {
                        self.state = AppState::Error(e);
                    }
                }
            }
            Message::SpecialtySelected(specialty) => {
                self.state = AppState::ScoreSelection { specialty };
            }
            Message::ScoreSelected(score_id) => {
                if let AppState::ScoreSelection { specialty } = self.state {
                    self.state = AppState::ScoreCalculation {
                        specialty,
                        score_id,
                        input_state: ScoreInputState::new(),
                        result: None,
                        error: None,
                    };
                }
            }
            Message::Input(input_msg) => {
                if let AppState::ScoreCalculation {
                    ref score_id,
                    ref mut input_state,
                    ref mut result,
                    ref mut error,
                    ..
                } = self.state
                {
                    match input_msg {
                        InputMessage::BooleanChanged(field, value) => {
                            input_state.update_boolean(field, value);
                            // Clear error when user makes changes
                            *error = None;
                        }
                        InputMessage::NumberTextChanged(field, value) => {
                            input_state.update_number_text(field, value);
                            // Clear error when user makes changes
                            *error = None;
                        }
                        InputMessage::DropdownSelected(field, value) => {
                            input_state.update_dropdown(field, value);
                            // Clear error when user makes changes
                            *error = None;
                        }
                        InputMessage::Calculate => {
                            // Perform calculation
                            if let Some(library) = &self.score_library {
                                if let Some(score_def) = library.get_score(score_id) {
                                    match calculate_score(score_def, &input_state.inputs) {
                                        Ok(calc_result) => {
                                            *result = Some(calc_result);
                                            *error = None;
                                        }
                                        Err(e) => {
                                            *result = None;
                                            *error = Some(format!("{}", e));
                                        }
                                    }
                                }
                            }
                        }
                        InputMessage::Reset => {
                            *input_state = ScoreInputState::new();
                            *result = None;
                            *error = None;
                        }
                    }
                }
            }
            Message::BackToWelcome => {
                self.state = AppState::Welcome;
            }
            Message::BackToSpecialtySelection => {
                self.state = AppState::SpecialtySelection;
            }
            Message::BackToScoreSelection => {
                if let AppState::ScoreCalculation { specialty, .. } = self.state {
                    self.state = AppState::ScoreSelection { specialty };
                }
            }
            Message::OpenSettings => {
                self.state = AppState::Settings;
            }
            Message::CloseSettings => {
                self.state = AppState::Welcome;
            }
            Message::ThemeChanged(theme) => {
                self.settings.theme = theme;
            }
        }
        Task::none()
    }

    fn theme(&self) -> iced::Theme {
        self.settings.theme.to_iced_theme()
    }

    fn view(&self) -> Element<'_, Message> {
        let language_label = match self.language {
            Language::German => "🇬🇧 English",
            Language::English => "🇩🇪 Deutsch",
        };

        let language_button = button(text(language_label))
            .on_press(Message::LanguageToggled)
            .padding(10);

        let settings_label = match self.language {
            Language::German => "⚙️ Einstellungen",
            Language::English => "⚙️ Settings",
        };

        let settings_button = button(text(settings_label))
            .on_press(Message::OpenSettings)
            .padding(10);

        let header = row![
            text("KlinScore")
                .size(32)
                .width(Length::Fill),
            settings_button,
            language_button
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding(20);

        let content = match &self.state {
            AppState::Loading => self.loading_view(),
            AppState::Welcome => self.welcome_view(),
            AppState::SpecialtySelection => self.specialty_view(),
            AppState::ScoreSelection { specialty } => self.score_selection_view(*specialty),
            AppState::ScoreCalculation {
                specialty,
                score_id,
                input_state,
                result,
                error,
            } => self.calculation_view(*specialty, score_id, input_state, result.as_ref(), error.as_deref()),
            AppState::Settings => self.settings_view(),
            AppState::Error(error) => self.error_view(error),
        };

        let scrollable_content = scrollable(content);

        let main_column = column![header, scrollable_content]
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill);

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(20)
            .into()
    }

    fn loading_view(&self) -> Element<'_, Message> {
        let message = match self.language {
            Language::German => "Lade Score-Bibliothek...",
            Language::English => "Loading score library...",
        };

        let content = column![text(message).size(24)]
            .align_x(Alignment::Center)
            .padding(50);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn welcome_view(&self) -> Element<'_, Message> {
        let title = match self.language {
            Language::German => "Willkommen bei KlinScore",
            Language::English => "Welcome to KlinScore",
        };

        let subtitle = match self.language {
            Language::German => {
                "Quelloffener klinischer Score-Rechner für evidenzbasierte Medizin"
            }
            Language::English => {
                "Open-source clinical score calculator for evidence-based medicine"
            }
        };

        let scores_loaded = match &self.score_library {
            Some(library) => {
                let msg = match self.language {
                    Language::German => format!("{} Scores geladen", library.count()),
                    Language::English => format!("{} scores loaded", library.count()),
                };
                text(msg).size(14)
            }
            None => text(""),
        };

        let start_button_label = match self.language {
            Language::German => "Score berechnen",
            Language::English => "Calculate Score",
        };

        let content = column![
            text(title).size(40),
            text(subtitle).size(16),
            scores_loaded,
            button(text(start_button_label).size(20))
                .on_press(Message::BackToSpecialtySelection)
                .padding(15),
        ]
        .spacing(30)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn specialty_view(&self) -> Element<'_, Message> {
        ui::specialty_selection_view(
            self.language,
            Message::SpecialtySelected,
            Message::BackToWelcome,
        )
    }

    fn score_selection_view(&self, specialty: Specialty) -> Element<'_, Message> {
        let title = match self.language {
            Language::German => format!("{} - Score auswählen", specialty.to_german()),
            Language::English => format!("{} - Select Score", specialty.to_english()),
        };

        let subtitle = match self.language {
            Language::German => "Wählen Sie einen Score zur Berechnung",
            Language::English => "Choose a score to calculate",
        };

        let back_label = match self.language {
            Language::German => "← Zurück zu Fachgebieten",
            Language::English => "← Back to Specialties",
        };

        // Get scores for this specialty
        let score_buttons: Element<Message> = if let Some(library) = &self.score_library {
            let scores = library.get_scores_for_specialty(specialty);

            if scores.is_empty() {
                let msg = match self.language {
                    Language::German => "Keine Scores für dieses Fachgebiet verfügbar",
                    Language::English => "No scores available for this specialty",
                };
                column![text(msg).size(18)]
                    .align_x(Alignment::Center)
                    .into()
            } else {
                let score_buttons_vec: Vec<Element<Message>> = scores
                    .into_iter()
                    .map(|score| {
                        let label = match self.language {
                            Language::German => &score.name_de,
                            Language::English => &score.name,
                        };

                        // Get score ID from library
                        let score_id = library
                            .scores
                            .iter()
                            .find(|(_, s)| s.name == score.name)
                            .map(|(id, _)| id.clone())
                            .unwrap_or_default();

                        button(
                            column![
                                text(label).size(20),
                                text(&score.guideline_source).size(14)
                            ]
                            .spacing(5),
                        )
                        .on_press(Message::ScoreSelected(score_id))
                        .padding(15)
                        .width(Length::Fixed(400.0))
                        .into()
                    })
                    .collect();

                column(score_buttons_vec)
                    .spacing(15)
                    .align_x(Alignment::Center)
                    .into()
            }
        } else {
            text("Loading...").into()
        };

        let content = column![
            text(title).size(32),
            text(subtitle).size(16),
            score_buttons,
            button(text(back_label).size(18))
                .on_press(Message::BackToSpecialtySelection)
                .padding(10),
        ]
        .spacing(25)
        .align_x(Alignment::Center)
        .padding(40);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn calculation_view<'a>(
        &'a self,
        _specialty: Specialty,
        score_id: &str,
        input_state: &'a ScoreInputState,
        result: Option<&'a CalculationResult>,
        error: Option<&'a str>,
    ) -> Element<'a, Message> {
        let score = self
            .score_library
            .as_ref()
            .and_then(|lib| lib.get_score(score_id));

        let back_label = match self.language {
            Language::German => "← Zurück zur Score-Auswahl",
            Language::English => "← Back to Score Selection",
        };

        if let Some(score_def) = score {
            // Show result if available, otherwise show input form
            if let Some(calc_result) = result {
                ui::result_display_view(
                    calc_result,
                    self.language,
                    Message::Input(InputMessage::Reset),
                    Message::BackToScoreSelection,
                )
            } else {
                let form = ui::score_input_form(score_def, input_state, self.language, |msg| {
                    Message::Input(msg)
                });

                let error_label = match self.language {
                    Language::German => "Fehler: ",
                    Language::English => "Error: ",
                };

                let mut content = vec![form];

                // Display error if present
                if let Some(err) = error {
                    let error_box = container(
                        text(format!("{}{}", error_label, err))
                            .size(16)
                            .color(iced::Color::from_rgb(0.8, 0.1, 0.1))
                    )
                    .padding(10)
                    .style(|_theme: &iced::Theme| {
                        container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(1.0, 0.9, 0.9))),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.8, 0.1, 0.1),
                                width: 2.0,
                                radius: 5.0.into(),
                            },
                            ..Default::default()
                        }
                    });
                    content.push(error_box.into());
                }

                content.push(
                    button(text(back_label).size(18))
                        .on_press(Message::BackToScoreSelection)
                        .padding(10)
                        .into()
                );

                column(content)
                    .spacing(20)
                    .align_x(Alignment::Center)
                    .into()
            }
        } else {
            text("Score not found").into()
        }
    }

    fn error_view<'a>(&self, error: &'a str) -> Element<'a, Message> {
        let title = match self.language {
            Language::German => "Fehler beim Laden",
            Language::English => "Loading Error",
        };

        let content = column![
            text(title).size(32),
            text(error).size(16),
            button(text("OK"))
                .on_press(Message::BackToWelcome)
                .padding(10),
        ]
        .spacing(20)
        .align_x(Alignment::Center)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn settings_view<'a>(&'a self) -> Element<'a, Message> {
        let title = match self.language {
            Language::German => "Einstellungen",
            Language::English => "Settings",
        };

        let theme_label = match self.language {
            Language::German => "Farbschema:",
            Language::English => "Theme:",
        };

        let back_label = match self.language {
            Language::German => "← Zurück",
            Language::English => "← Back",
        };

        let theme_picker = pick_list(
            AppTheme::all(),
            Some(self.settings.theme),
            Message::ThemeChanged,
        )
        .placeholder("Select theme")
        .width(Length::Fixed(200.0));

        let theme_display = match self.language {
            Language::German => self.settings.theme.to_german(),
            Language::English => self.settings.theme.to_string(),
        };

        let content = column![
            text(title).size(32),
            column![
                text(theme_label).size(18),
                theme_picker,
                text(format!("Current: {}", theme_display)).size(14),
            ]
            .spacing(10)
            .padding(20),
            button(text(back_label).size(18))
                .on_press(Message::CloseSettings)
                .padding(10),
        ]
        .spacing(30)
        .align_x(Alignment::Center)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }
}
