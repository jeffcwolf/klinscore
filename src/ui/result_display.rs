// result_display.rs
// Display calculation results with risk visualization

use crate::config::RiskLevel;
use crate::scores::CalculationResult;
use crate::ui::Language;
use iced::{
    widget::{button, column, container, row, text},
    Alignment, Color, Element, Length,
};

/// Display calculation result with color-coded risk
pub fn result_display_view<'a, Message>(
    result: &'a CalculationResult,
    language: Language,
    on_recalculate: Message,
    on_back: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let score_label = match language {
        Language::German => "Score:",
        Language::English => "Score:",
    };

    let risk_label = match language {
        Language::German => "Risiko:",
        Language::English => "Risk:",
    };

    let recommendation_label = match language {
        Language::German => "Empfehlung:",
        Language::English => "Recommendation:",
    };

    let risk_text = match language {
        Language::German => &result.risk_de,
        Language::English => &result.risk,
    };

    let recommendation_text = match language {
        Language::German => &result.recommendation_de,
        Language::English => &result.recommendation,
    };

    let details_text = match language {
        Language::German => result.details_de.as_ref(),
        Language::English => result.details.as_ref(),
    };

    // Get color for risk level
    let risk_color = get_risk_color(result.risk_level);

    let recalculate_label = match language {
        Language::German => "Neu berechnen",
        Language::English => "Calculate Again",
    };

    let back_label = match language {
        Language::German => "← Zurück",
        Language::English => "← Back",
    };

    let mut content_widgets = vec![
        // Score value - large and prominent
        text(format!("{} {}", score_label, result.total_score))
            .size(48)
            .into(),
        // Risk level with color
        container(
            text(format!("{} {}", risk_label, risk_text))
                .size(32)
                .color(risk_color),
        )
        .padding(15)
        .into(),
        // Recommendation
        column![
            text(recommendation_label).size(18),
            text(recommendation_text).size(16),
        ]
        .spacing(5)
        .padding(15)
        .into(),
    ];

    // Add details if present
    if let Some(details) = details_text {
        content_widgets.push(
            column![
                text("Details:").size(14),
                text(details).size(12),
            ]
            .spacing(5)
            .padding(10)
            .into(),
        );
    }

    // Add breakdown of points
    if !result.field_points.is_empty() {
        let breakdown_label = match language {
            Language::German => "Punkteverteilung:",
            Language::English => "Points Breakdown:",
        };

        let breakdown_items: Vec<Element<'a, Message>> = result
            .field_points
            .iter()
            .map(|(field, points)| {
                text(format!("  • {}: {} points", field, points))
                    .size(14)
                    .into()
            })
            .collect();

        content_widgets.push(
            column![text(breakdown_label).size(14), column(breakdown_items).spacing(3)]
                .spacing(5)
                .padding(10)
                .into(),
        );
    }

    // Add action buttons
    content_widgets.push(
        row![
            button(text(recalculate_label).size(18))
                .on_press(on_recalculate)
                .padding(12),
            button(text(back_label).size(18))
                .on_press(on_back)
                .padding(12),
        ]
        .spacing(15)
        .padding(20)
        .into(),
    );

    let content = column(content_widgets)
        .spacing(20)
        .align_x(Alignment::Center)
        .padding(30)
        .max_width(700);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Get color for risk level
fn get_risk_color(level: RiskLevel) -> Color {
    let (r, g, b) = level.rgb();
    Color::from_rgb(r, g, b)
}
