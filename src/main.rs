// KlinScore - Clinical Score Calculator
// main.rs

use iced::{
    widget::{button, column, container, row, text},
    alignment::{Horizontal, Vertical},
    Alignment, Element, Length, Task,
};

fn main() -> iced::Result {
    iced::application("KlinScore", KlinScore::update, KlinScore::view)
        .window_size((1000.0, 700.0))
        .run_with(KlinScore::new)
}

// Application State
#[derive(Debug, Clone)]
enum AppState {
    Welcome,
    SpecialtySelection,
    ScoreCalculation,
}

// Main Application
struct KlinScore {
    state: AppState,
    language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    German,
    English,
}

// Messages (user interactions)
#[derive(Debug, Clone)]
enum Message {
    LanguageToggled,
    StateChanged(AppState),
}

impl KlinScore {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState::Welcome,
                language: Language::German,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LanguageToggled => {
                self.language = match self.language {
                    Language::German => Language::English,
                    Language::English => Language::German,
                };
            }
            Message::StateChanged(new_state) => {
                self.state = new_state;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let language_label = match self.language {
            Language::German => "🇬🇧 English",
            Language::English => "🇩🇪 Deutsch",
        };

        let language_button = button(text(language_label))
            .on_press(Message::LanguageToggled)
            .padding(10);

        let header = row![
            text("KlinScore")
                .size(32)
                .width(Length::Fill),
            language_button
        ]
        .align_y(Alignment::Center)
        .padding(20);

        let content = match self.state {
            AppState::Welcome => self.welcome_view(),
            AppState::SpecialtySelection => self.specialty_view(),
            AppState::ScoreCalculation => self.calculation_view(),
        };

        let main_column = column![header, content]
            .spacing(20)
            .padding(20)
            .width(Length::Fill);

        container(main_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn welcome_view(&self) -> Element<Message> {
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

        let start_button_label = match self.language {
            Language::German => "Score berechnen",
            Language::English => "Calculate Score",
        };

        let content = column![
            text(title).size(40),
            text(subtitle).size(16),
            button(text(start_button_label).size(20))
                .on_press(Message::StateChanged(AppState::SpecialtySelection))
                .padding(15),
        ]
        .spacing(30)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn specialty_view(&self) -> Element<Message> {
        let title = match self.language {
            Language::German => "Fachgebiet auswählen",
            Language::English => "Select Specialty",
        };

        let content = column![
            text(title).size(28),
            text("Coming soon: Cardiology, Nephrology, Anesthesiology...").size(16),
            button(text("← Back"))
                .on_press(Message::StateChanged(AppState::Welcome))
                .padding(10),
        ]
        .spacing(20)
        .align_x(Alignment::Center)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn calculation_view(&self) -> Element<Message> {
        let content = column![
            text("Score Calculation").size(28),
            text("Score calculator will go here...").size(16),
            button(text("← Back"))
                .on_press(Message::StateChanged(AppState::SpecialtySelection))
                .padding(10),
        ]
        .spacing(20)
        .align_x(Alignment::Center)
        .padding(50);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}