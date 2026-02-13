// KlinScore - Clinical Score Calculator
// main.rs

mod config;
mod export;
mod persistence;
mod scores;
mod settings;
mod ui;

use config::Specialty;
use export::ExportRecord;
use scores::{calculate_score, load_all_scores, CalculationResult, ScoreLibrary};
use settings::{AppTheme, Settings};
use ui::{InputMessage, Language, ScoreInputState};

use chrono::Local;
use iced::{
    widget::{button, column, container, horizontal_rule, pick_list, row, scrollable, text},
    Alignment, Element, Length, Task,
};

/// A single calculation history entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct HistoryEntry {
    score_name: String,
    score_name_de: String,
    score_id: String,
    specialty: Specialty,
    total_score: i32,
    risk: String,
    risk_de: String,
    timestamp: String,
}

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
        result: Option<Box<CalculationResult>>,
        error: Option<String>,
    },
    History,
    About,
    Settings,
    Error(String),
}

// Main Application
struct KlinScore {
    state: AppState,
    language: Language,
    score_library: Option<ScoreLibrary>,
    settings: Settings,
    history: Vec<HistoryEntry>,
    /// Tracks the previous state to return to from About/History
    previous_state: Option<Box<AppState>>,
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
    OpenHistory,
    CloseHistory,
    ClearHistory,
    OpenAbout,
    CloseAbout,
    OpenUrl(String),
    ExportCsv,
    ExportJson,
    ExportPdf,
    ExportComplete(Result<String, String>),
}

impl KlinScore {
    fn new() -> (Self, Task<Message>) {
        // Load persisted settings and history
        let (settings, language) = match persistence::load_settings() {
            Some(persisted) => {
                let mut settings = Settings::new();
                settings.theme = persisted.theme;
                settings.show_help_hints = persisted.show_help_hints;
                settings.auto_calculate = persisted.auto_calculate;
                (settings, persisted.language)
            }
            None => (Settings::new(), Language::German),
        };
        let history: Vec<HistoryEntry> = persistence::load_history();

        let app = Self {
            state: AppState::Loading,
            language,
            score_library: None,
            settings,
            history,
            previous_state: None,
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
                persistence::save_settings(&self.settings, self.language);
            }
            Message::ScoresLoaded(result) => match result {
                Ok(library) => {
                    self.score_library = Some(library);
                    self.state = AppState::Welcome;
                }
                Err(e) => {
                    self.state = AppState::Error(e);
                }
            },
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
                    specialty,
                    ref score_id,
                    ref mut input_state,
                    ref mut result,
                    ref mut error,
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
                                            // Save to history
                                            let entry = HistoryEntry {
                                                score_name: score_def.name.clone(),
                                                score_name_de: score_def.name_de.clone(),
                                                score_id: score_id.clone(),
                                                specialty,
                                                total_score: calc_result.total_score,
                                                risk: calc_result.risk.clone(),
                                                risk_de: calc_result.risk_de.clone(),
                                                timestamp: Local::now()
                                                    .format("%Y-%m-%d %H:%M")
                                                    .to_string(),
                                            };
                                            self.history.push(entry);
                                            persistence::save_history(&self.history);

                                            *result = Some(Box::new(calc_result));
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
                persistence::save_settings(&self.settings, self.language);
            }
            Message::OpenHistory => {
                self.previous_state = Some(Box::new(self.state.clone()));
                self.state = AppState::History;
            }
            Message::CloseHistory => {
                self.state = self
                    .previous_state
                    .take()
                    .map(|s| *s)
                    .unwrap_or(AppState::Welcome);
            }
            Message::ClearHistory => {
                self.history.clear();
                persistence::save_history(&self.history);
            }
            Message::OpenAbout => {
                self.previous_state = Some(Box::new(self.state.clone()));
                self.state = AppState::About;
            }
            Message::CloseAbout => {
                self.state = self
                    .previous_state
                    .take()
                    .map(|s| *s)
                    .unwrap_or(AppState::Welcome);
            }
            Message::OpenUrl(url) => {
                let _ = opener::open(&url);
            }
            Message::ExportCsv => {
                if let Some(record) = self.current_export_record() {
                    let filename = export::default_filename(&record.score_name, "csv");
                    return Task::perform(
                        async move {
                            export::csv_export::export_to_csv_file(&record, &filename)
                                .map(|()| filename)
                        },
                        Message::ExportComplete,
                    );
                }
            }
            Message::ExportJson => {
                if let Some(record) = self.current_export_record() {
                    let filename = export::default_filename(&record.score_name, "json");
                    return Task::perform(
                        async move {
                            export::json_export::export_to_json_file(&record, &filename)
                                .map(|()| filename)
                        },
                        Message::ExportComplete,
                    );
                }
            }
            Message::ExportPdf => {
                if let Some(record) = self.current_export_record() {
                    let filename = export::default_filename(&record.score_name, "pdf");
                    return Task::perform(
                        async move {
                            export::pdf_export::export_to_pdf_file(&record, &filename)
                                .map(|()| filename)
                        },
                        Message::ExportComplete,
                    );
                }
            }
            Message::ExportComplete(result) => {
                if let AppState::ScoreCalculation { ref mut error, .. } = self.state {
                    match result {
                        Ok(filename) => {
                            let msg = match self.language {
                                Language::German => format!("Exportiert: {}", filename),
                                Language::English => format!("Exported: {}", filename),
                            };
                            *error = Some(msg); // Reuse error field for status messages
                        }
                        Err(e) => {
                            let msg = match self.language {
                                Language::German => format!("Export fehlgeschlagen: {}", e),
                                Language::English => format!("Export failed: {}", e),
                            };
                            *error = Some(msg);
                        }
                    }
                }
            }
        }
        Task::none()
    }

    /// Build an ExportRecord from the current calculation result (if any)
    fn current_export_record(&self) -> Option<ExportRecord> {
        if let AppState::ScoreCalculation {
            ref score_id,
            ref result,
            ..
        } = self.state
        {
            let calc_result = result.as_ref()?;
            let score_def = self
                .score_library
                .as_ref()
                .and_then(|lib| lib.get_score(score_id))?;
            let score_name = match self.language {
                Language::German => &score_def.name_de,
                Language::English => &score_def.name,
            };
            let use_german = self.language == Language::German;
            Some(ExportRecord::from_result(
                calc_result,
                score_name,
                use_german,
            ))
        } else {
            None
        }
    }

    fn theme(&self) -> iced::Theme {
        self.settings.theme.iced_theme()
    }

    fn view(&self) -> Element<'_, Message> {
        let language_label = match self.language {
            Language::German => "🇬🇧 English",
            Language::English => "🇩🇪 Deutsch",
        };

        let language_button = button(text(language_label))
            .on_press(Message::LanguageToggled)
            .padding(10);

        let history_label = match self.language {
            Language::German => "Verlauf",
            Language::English => "History",
        };

        let history_count = if self.history.is_empty() {
            String::new()
        } else {
            format!(" ({})", self.history.len())
        };

        let history_button = button(text(format!("{}{}", history_label, history_count)))
            .on_press(Message::OpenHistory)
            .padding(10);

        let about_label = match self.language {
            Language::German => "Über",
            Language::English => "About",
        };

        let about_button = button(text(about_label))
            .on_press(Message::OpenAbout)
            .padding(10);

        let settings_label = match self.language {
            Language::German => "Einstellungen",
            Language::English => "Settings",
        };

        let settings_button = button(text(settings_label))
            .on_press(Message::OpenSettings)
            .padding(10);

        let header = row![
            text("KlinScore").size(32).width(Length::Fill),
            history_button,
            settings_button,
            about_button,
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
            } => self.calculation_view(
                *specialty,
                score_id,
                input_state,
                result.as_deref(),
                error.as_deref(),
            ),
            AppState::History => self.history_view(),
            AppState::About => self.about_view(),
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
            Language::German => "Quelloffener klinischer Score-Rechner für evidenzbasierte Medizin",
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
            Language::German => format!("{} - Score auswählen", specialty.german()),
            Language::English => format!("{} - Select Score", specialty.english()),
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
                            column![text(label).size(20), text(&score.guideline_source).size(14)]
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
                    Message::ExportCsv,
                    Message::ExportJson,
                    Message::ExportPdf,
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
                            .color(iced::Color::from_rgb(0.8, 0.1, 0.1)),
                    )
                    .padding(10)
                    .style(|_theme: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            1.0, 0.9, 0.9,
                        ))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.8, 0.1, 0.1),
                            width: 2.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    });
                    content.push(error_box.into());
                }

                content.push(
                    button(text(back_label).size(18))
                        .on_press(Message::BackToScoreSelection)
                        .padding(10)
                        .into(),
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

    fn history_view(&self) -> Element<'_, Message> {
        let title = match self.language {
            Language::German => "Berechnungsverlauf",
            Language::English => "Calculation History",
        };

        let back_label = match self.language {
            Language::German => "← Zurück",
            Language::English => "← Back",
        };

        let mut content_widgets: Vec<Element<'_, Message>> = vec![text(title).size(32).into()];

        if self.history.is_empty() {
            let empty_msg = match self.language {
                Language::German => "Noch keine Berechnungen durchgeführt.",
                Language::English => "No calculations yet.",
            };
            content_widgets.push(text(empty_msg).size(16).into());
        } else {
            let clear_label = match self.language {
                Language::German => "Verlauf löschen",
                Language::English => "Clear History",
            };

            content_widgets.push(
                row![
                    text(format!(
                        "{} {}",
                        self.history.len(),
                        match self.language {
                            Language::German => "Berechnungen",
                            Language::English => "calculations",
                        }
                    ))
                    .size(14)
                    .width(Length::Fill),
                    button(text(clear_label).size(14))
                        .on_press(Message::ClearHistory)
                        .padding(8),
                ]
                .align_y(Alignment::Center)
                .into(),
            );

            content_widgets.push(horizontal_rule(1).into());

            // Show history entries in reverse chronological order
            for entry in self.history.iter().rev() {
                let score_name = match self.language {
                    Language::German => &entry.score_name_de,
                    Language::English => &entry.score_name,
                };

                let risk_text = match self.language {
                    Language::German => &entry.risk_de,
                    Language::English => &entry.risk,
                };

                let specialty_text = match self.language {
                    Language::German => entry.specialty.german(),
                    Language::English => entry.specialty.english(),
                };

                let entry_widget = container(
                    column![
                        row![
                            text(score_name).size(18).width(Length::Fill),
                            text(&entry.timestamp).size(12),
                        ]
                        .align_y(Alignment::Center),
                        row![
                            text(format!(
                                "{}: {}",
                                match self.language {
                                    Language::German => "Score",
                                    Language::English => "Score",
                                },
                                entry.total_score
                            ))
                            .size(14),
                            text(" | ").size(14),
                            text(risk_text).size(14),
                            text(" | ").size(14),
                            text(specialty_text).size(12),
                        ]
                        .spacing(5),
                    ]
                    .spacing(5),
                )
                .padding(12)
                .width(Length::Fill)
                .style(|theme: &iced::Theme| {
                    let palette = theme.palette();
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color {
                            a: 0.05,
                            ..palette.text
                        })),
                        border: iced::Border {
                            color: iced::Color {
                                a: 0.15,
                                ..palette.text
                            },
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    }
                });

                content_widgets.push(entry_widget.into());
            }
        }

        content_widgets.push(
            button(text(back_label).size(18))
                .on_press(Message::CloseHistory)
                .padding(10)
                .into(),
        );

        let content = column(content_widgets)
            .spacing(15)
            .align_x(Alignment::Center)
            .padding(40)
            .max_width(700);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn about_view(&self) -> Element<'_, Message> {
        let is_de = self.language == Language::German;

        let back_label = if is_de { "← Zurück" } else { "← Back" };

        let scores_count = self
            .score_library
            .as_ref()
            .map(|lib| lib.count())
            .unwrap_or(0);

        let title = if is_de {
            "Über KlinScore"
        } else {
            "About KlinScore"
        };
        let subtitle = if is_de {
            "Quelloffener klinischer Score-Rechner für evidenzbasierte Medizin"
        } else {
            "Open-source clinical score calculator for evidence-based medicine"
        };

        // -- Version / stats --
        let version_label = "Version";
        let scores_label = if is_de {
            "Scores geladen"
        } else {
            "Scores loaded"
        };
        let stats_section = column![
            text(format!("{}: 0.1.0", version_label)).size(14),
            text(format!("{}: {}", scores_label, scores_count)).size(14),
            text(if is_de {
                "Fachgebiete: Kardiologie, Nephrologie, Anästhesiologie"
            } else {
                "Specialties: Cardiology, Nephrology, Anesthesiology"
            })
            .size(14),
            text(if is_de {
                "Leitlinien: ESC 2024, KDIGO, ACCP, ASA"
            } else {
                "Guidelines: ESC 2024, KDIGO, ACCP, ASA"
            })
            .size(14),
            text("Lizenz / License: MIT / Apache 2.0").size(14),
        ]
        .spacing(5)
        .padding(10);

        // -- Disclaimer --
        let disclaimer_title = if is_de {
            "Haftungsausschluss"
        } else {
            "Disclaimer"
        };
        let disclaimer_text = if is_de {
            "KlinScore ist ein Hilfsmittel zur klinischen Entscheidungsunterstützung. \
             Es ersetzt nicht die klinische Beurteilung durch einen Arzt. \
             Alle Scores sollten im klinischen Kontext des Patienten interpretiert werden. \
             Keine Garantie für Richtigkeit oder Vollständigkeit."
        } else {
            "KlinScore is a clinical decision support tool. \
             It does not replace clinical judgment by a physician. \
             All scores should be interpreted in the patient's clinical context. \
             No guarantee of accuracy or completeness."
        };

        let disclaimer_box = container(
            column![
                text(disclaimer_title).size(16),
                text(disclaimer_text).size(13),
            ]
            .spacing(5),
        )
        .padding(15)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.08,
                    ..palette.danger
                })),
                border: iced::Border {
                    color: iced::Color {
                        a: 0.3,
                        ..palette.danger
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        });

        // -- Score Methodology & Sources (main transparency section) --
        let methodology_section = self.about_methodology_section();

        let content = column![
            text(title).size(32),
            text(subtitle).size(16),
            horizontal_rule(1),
            stats_section,
            horizontal_rule(1),
            disclaimer_box,
            horizontal_rule(1),
            methodology_section,
            horizontal_rule(1),
            // Back button
            button(text(back_label).size(18))
                .on_press(Message::CloseAbout)
                .padding(10),
        ]
        .spacing(15)
        .align_x(Alignment::Center)
        .padding(40)
        .max_width(900);

        container(content)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    /// Build the methodology & sources section for the About page.
    /// Shows each score with: calculation method, inputs, formula/logic, and clickable reference.
    fn about_methodology_section(&self) -> Element<'_, Message> {
        let is_de = self.language == Language::German;

        let section_title = if is_de {
            "Score-Methodik & Quellenangaben"
        } else {
            "Score Methodology & Sources"
        };

        let section_subtitle = if is_de {
            "Vollständige Transparenz über die verwendeten Berechnungsverfahren und deren wissenschaftliche Grundlage."
        } else {
            "Full transparency on calculation methods and their scientific basis."
        };

        let library = match &self.score_library {
            Some(lib) => lib,
            None => return text("").into(),
        };

        let mut all_cards: Vec<Element<'_, Message>> = Vec::new();

        for specialty in &[
            Specialty::Cardiology,
            Specialty::Nephrology,
            Specialty::Anesthesiology,
        ] {
            let scores = library.get_scores_for_specialty(*specialty);
            if scores.is_empty() {
                continue;
            }

            let specialty_name = if is_de {
                specialty.german()
            } else {
                specialty.english()
            };

            // Specialty header
            all_cards.push(text(format!("--- {} ---", specialty_name)).size(18).into());

            for score in &scores {
                all_cards.push(self.score_methodology_card(score));
            }
        }

        column![
            text(section_title).size(24),
            text(section_subtitle).size(14),
            column(all_cards).spacing(12),
        ]
        .spacing(12)
        .padding(10)
        .into()
    }

    /// Build a single score methodology card with: name, method, inputs, calculation, reference link.
    fn score_methodology_card<'a>(
        &self,
        score: &'a config::ScoreDefinition,
    ) -> Element<'a, Message> {
        let is_de = self.language == Language::German;

        let name = if is_de { &score.name_de } else { &score.name };
        let description = if is_de {
            &score.description_de
        } else {
            &score.description
        };

        // Determine calculation method
        let (method_label, method_detail) = if let Some(ref formula) = score.formula {
            let label = if is_de {
                "Formelbasiert"
            } else {
                "Formula-based"
            };
            let detail = match formula.as_str() {
                "ckd_epi_2021" => {
                    if is_de {
                        "eGFR = 142 \u{00d7} min(Scr/\u{03ba}, 1)^\u{03b1} \u{00d7} max(Scr/\u{03ba}, 1)^(\u{2212}1.200) \u{00d7} 0.9938^Alter \u{00d7} (1.012 bei Frauen)\n\
                         Wobei: Weiblich \u{03ba}=0.7, \u{03b1}=\u{2212}0.241 | Männlich \u{03ba}=0.9, \u{03b1}=\u{2212}0.302\n\
                         Scr in mg/dL (Eingabe \u{03bc}mol/L \u{00f7} 88.4)"
                            .to_string()
                    } else {
                        "eGFR = 142 \u{00d7} min(Scr/\u{03ba}, 1)^\u{03b1} \u{00d7} max(Scr/\u{03ba}, 1)^(\u{2212}1.200) \u{00d7} 0.9938^age \u{00d7} (1.012 if female)\n\
                         Where: Female \u{03ba}=0.7, \u{03b1}=\u{2212}0.241 | Male \u{03ba}=0.9, \u{03b1}=\u{2212}0.302\n\
                         Scr in mg/dL (input \u{03bc}mol/L \u{00f7} 88.4)"
                            .to_string()
                    }
                }
                "kfre_4var" => {
                    if is_de {
                        "Risiko = 1 \u{2212} 0.9832^exp(Summe)\n\
                         Summe = \u{2212}0.2201\u{00d7}(Alter/10 \u{2212} 7.036) + 0.2467\u{00d7}(männl. \u{2212} 0.5642) \u{2212} 0.5567\u{00d7}(eGFR/5 \u{2212} 7.222) + 0.4510\u{00d7}(ln(ACR) \u{2212} 5.137)\n\
                         ACR-Eingabe: mg/mmol, intern umgerechnet in mg/g (\u{00d7} 8.84)"
                            .to_string()
                    } else {
                        "Risk = 1 \u{2212} 0.9832^exp(sum)\n\
                         sum = \u{2212}0.2201\u{00d7}(age/10 \u{2212} 7.036) + 0.2467\u{00d7}(male \u{2212} 0.5642) \u{2212} 0.5567\u{00d7}(eGFR/5 \u{2212} 7.222) + 0.4510\u{00d7}(ln(ACR) \u{2212} 5.137)\n\
                         ACR input: mg/mmol, converted to mg/g internally (\u{00d7} 8.84)"
                            .to_string()
                    }
                }
                _ => formula.clone(),
            };
            (label, detail)
        } else {
            let label = if is_de {
                "Punktebasiert"
            } else {
                "Point-based"
            };
            let detail = if is_de {
                "Summe der Punkte aller zutreffenden Kriterien".to_string()
            } else {
                "Sum of points for all applicable criteria".to_string()
            };
            (label, detail)
        };

        // Build inputs summary
        let inputs_label = if is_de { "Eingaben" } else { "Inputs" };
        let inputs_summary: String = score
            .inputs
            .iter()
            .map(|input| {
                let label = if is_de { &input.label_de } else { &input.label };
                let type_str = match input.input_type {
                    config::InputType::Boolean => {
                        if is_de {
                            "Ja/Nein"
                        } else {
                            "Yes/No"
                        }
                    }
                    config::InputType::Number => {
                        if let Some(ref unit) = input.unit {
                            return format!("{} ({})", label, unit);
                        }
                        if is_de {
                            "Zahl"
                        } else {
                            "Number"
                        }
                    }
                    config::InputType::Dropdown => {
                        if is_de {
                            "Auswahl"
                        } else {
                            "Selection"
                        }
                    }
                };
                format!("{} ({})", label, type_str)
            })
            .collect::<Vec<_>>()
            .join(", ");

        // Build the reference row with a clickable link button
        let reference_elements: Element<'_, Message> = if score.reference_url.is_empty() {
            text(&score.reference).size(12).into()
        } else {
            let url = score.reference_url.clone();
            let link_label = if is_de {
                "Quelle öffnen"
            } else {
                "Open source"
            };
            column![
                text(&score.reference).size(12),
                button(text(format!("\u{1f517} {}", link_label)).size(12))
                    .on_press(Message::OpenUrl(url))
                    .padding(4),
            ]
            .spacing(4)
            .into()
        };

        let method_heading = if is_de { "Methode" } else { "Method" };
        let calc_heading = if is_de { "Berechnung" } else { "Calculation" };
        let ref_heading = if is_de { "Referenz" } else { "Reference" };

        // Build the card
        let card = container(
            column![
                // Score name and description
                text(name).size(16),
                text(description).size(13),
                // Method
                row![
                    text(format!("{}: ", method_heading)).size(13),
                    text(method_label).size(13),
                ]
                .spacing(4),
                // Inputs
                text(format!("{}: {}", inputs_label, inputs_summary)).size(12),
                // Calculation detail
                container(
                    column![
                        text(format!("{}:", calc_heading)).size(13),
                        text(method_detail).size(12),
                    ]
                    .spacing(2),
                )
                .padding(8)
                .style(|theme: &iced::Theme| {
                    let palette = theme.palette();
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color {
                            a: 0.05,
                            ..palette.text
                        })),
                        border: iced::Border {
                            color: iced::Color::TRANSPARENT,
                            width: 0.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                }),
                // Reference
                column![
                    text(format!("{} ({})", ref_heading, score.guideline_source)).size(13),
                    reference_elements,
                ]
                .spacing(2),
            ]
            .spacing(6),
        )
        .padding(12)
        .style(|theme: &iced::Theme| {
            let palette = theme.palette();
            container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.04,
                    ..palette.text
                })),
                border: iced::Border {
                    color: iced::Color {
                        a: 0.15,
                        ..palette.text
                    },
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        });

        card.into()
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
            Language::German => self.settings.theme.german(),
            Language::English => self.settings.theme.label(),
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
