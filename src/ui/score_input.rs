// score_input.rs
// Dynamic form generator for score inputs

use crate::config::{InputField, InputType, ScoreDefinition};
use crate::scores::InputValue;
use crate::ui::Language;
use iced::{
    widget::{button, checkbox, column, container, row, text, text_input},
    Element, Length,
};
use std::collections::HashMap;

/// State for score input form
#[derive(Debug, Clone)]
pub struct ScoreInputState {
    pub inputs: HashMap<String, InputValue>,
    pub text_buffers: HashMap<String, String>,
}

impl ScoreInputState {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            text_buffers: HashMap::new(),
        }
    }

    pub fn update_boolean(&mut self, field: String, value: bool) {
        self.inputs.insert(field, InputValue::Boolean(value));
    }

    pub fn update_number_text(&mut self, field: String, value: String) {
        self.text_buffers.insert(field.clone(), value.clone());

        // Try to parse as number
        if let Ok(num) = value.parse::<f64>() {
            self.inputs.insert(field, InputValue::Number(num));
        }
    }

    pub fn update_dropdown(&mut self, field: String, value: String) {
        self.inputs.insert(field, InputValue::Dropdown(value));
    }
}

/// Messages for score input interactions
#[derive(Debug, Clone)]
pub enum InputMessage {
    BooleanChanged(String, bool),
    NumberTextChanged(String, String),
    #[allow(dead_code)]
    DropdownSelected(String, String),
    Calculate,
    Reset,
}

/// Generate dynamic input form for a score
pub fn score_input_form<'a, Message>(
    score: &'a ScoreDefinition,
    state: &'a ScoreInputState,
    language: Language,
    on_message: impl Fn(InputMessage) -> Message + 'a + Copy,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let title = match language {
        Language::German => &score.name_de,
        Language::English => &score.name,
    };

    let description = match language {
        Language::German => &score.description_de,
        Language::English => &score.description,
    };

    // Generate input fields
    let input_widgets: Vec<Element<'a, Message>> = score
        .inputs
        .iter()
        .map(|input_field| generate_input_widget(input_field, state, language, on_message))
        .collect();

    let calculate_label = match language {
        Language::German => "Berechnen",
        Language::English => "Calculate",
    };

    let reset_label = match language {
        Language::German => "Zurücksetzen",
        Language::English => "Reset",
    };

    let form_content = column![
        text(title).size(28),
        text(description).size(14),
        column(input_widgets).spacing(15).padding(20),
        row![
            button(text(calculate_label).size(18))
                .on_press(on_message(InputMessage::Calculate))
                .padding(12),
            button(text(reset_label).size(16))
                .on_press(on_message(InputMessage::Reset))
                .padding(12),
        ]
        .spacing(15),
    ]
    .spacing(20)
    .padding(20)
    .max_width(600);

    container(form_content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Generate a single input widget based on field type
fn generate_input_widget<'a, Message>(
    field: &'a InputField,
    state: &'a ScoreInputState,
    language: Language,
    on_message: impl Fn(InputMessage) -> Message + 'a + Copy,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let label_text = match language {
        Language::German => &field.label_de,
        Language::English => &field.label,
    };

    let unit_text = field.unit.as_ref().and_then(|_| {
        match language {
            Language::German => field.unit_de.as_ref().or(field.unit.as_ref()),
            Language::English => field.unit.as_ref(),
        }
    });

    let label_with_unit = if let Some(unit) = unit_text {
        format!("{} ({})", label_text, unit)
    } else {
        label_text.to_string()
    };

    match field.input_type {
        InputType::Boolean => {
            let is_checked = state
                .inputs
                .get(&field.field)
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let field_name = field.field.clone();
            let cb = checkbox(label_with_unit, is_checked).on_toggle(move |checked| {
                on_message(InputMessage::BooleanChanged(field_name.clone(), checked))
            });

            container(cb).padding(10).into()
        }

        InputType::Number => {
            let text_value = state
                .text_buffers
                .get(&field.field)
                .cloned()
                .unwrap_or_default();

            let placeholder = if let (Some(min), Some(max)) = (field.min, field.max) {
                format!("{} - {}", min, max)
            } else if let Some(min) = field.min {
                format!(">= {}", min)
            } else if let Some(max) = field.max {
                format!("<= {}", max)
            } else {
                String::new()
            };

            let field_name = field.field.clone();
            let input = text_input(&placeholder, &text_value)
                .on_input(move |value| {
                    on_message(InputMessage::NumberTextChanged(field_name.clone(), value))
                })
                .padding(8)
                .width(Length::Fixed(200.0));

            column![
                text(label_with_unit).size(16),
                input,
            ]
            .spacing(5)
            .padding(10)
            .into()
        }

        InputType::Dropdown => {
            // For now, show placeholder - full dropdown implementation would use iced::pick_list
            let current = state
                .inputs
                .get(&field.field)
                .and_then(|v| v.as_string())
                .unwrap_or("(select)");

            column![
                text(label_with_unit).size(16),
                text(format!("Selected: {}", current)).size(14),
                text("Dropdown options coming soon...").size(12),
            ]
            .spacing(5)
            .padding(10)
            .into()
        }
    }
}
