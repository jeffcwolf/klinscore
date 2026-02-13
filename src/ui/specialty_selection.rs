// specialty_selection.rs
// UI for selecting medical specialty

use crate::config::Specialty;
use iced::{
    widget::{button, column, container, text},
    Alignment, Element, Length,
};

/// Create the specialty selection view
pub fn specialty_selection_view<'a, Message>(
    language: Language,
    on_select: impl Fn(Specialty) -> Message + 'a,
    on_back: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let title = match language {
        Language::German => "Fachgebiet auswählen",
        Language::English => "Select Medical Specialty",
    };

    let subtitle = match language {
        Language::German => "Wählen Sie das Fachgebiet für die Score-Berechnung",
        Language::English => "Choose the medical specialty for score calculation",
    };

    // Specialty buttons
    let specialties = vec![
        (Specialty::Cardiology, "🫀", "Cardiology", "Kardiologie"),
        (Specialty::Nephrology, "🩺", "Nephrology", "Nephrologie"),
        (
            Specialty::Anesthesiology,
            "💉",
            "Anesthesiology",
            "Anästhesiologie",
        ),
    ];

    let buttons: Vec<Element<'a, Message>> = specialties
        .into_iter()
        .map(|(specialty, icon, name_en, name_de)| {
            let label = match language {
                Language::German => format!("{} {}", icon, name_de),
                Language::English => format!("{} {}", icon, name_en),
            };

            button(text(label).size(24))
                .on_press(on_select(specialty))
                .padding(20)
                .width(Length::Fixed(350.0))
                .into()
        })
        .collect();

    let specialty_buttons: Element<'a, Message> = column(buttons)
        .spacing(15)
        .align_x(Alignment::Center)
        .into();

    let back_label = match language {
        Language::German => "← Zurück",
        Language::English => "← Back",
    };

    let content = column![
        text(title).size(36),
        text(subtitle).size(16),
        specialty_buttons,
        button(text(back_label).size(18))
            .on_press(on_back)
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

/// Language setting for UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    German,
    English,
}
