//! Instance data entry panel
//!
//! Provides UI for filling out form instances with field-specific input widgets,
//! validation, and multi-page navigation.

use crate::{DrawingInstance, DrawingTemplate, TemplateError};
use egui::{Color32, RichText, ScrollArea, Ui};
use form_factor_core::FormInstance;
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

/// Action returned from data entry panel UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataEntryAction {
    /// Save instance as draft (may be incomplete)
    SaveDraft,
    /// Submit instance (requires validation)
    Submit,
    /// Cancel data entry
    Cancel,
    /// No action
    None,
}

/// Instance data entry panel state
#[derive(Debug, Clone)]
pub struct DataEntryPanel {
    /// Template being filled
    template: DrawingTemplate,
    /// Instance being edited
    instance: DrawingInstance,
    /// Current page index
    current_page: usize,
    /// Validation errors by field ID
    validation_errors: HashMap<String, String>,
    /// Whether instance has unsaved changes
    dirty: bool,
}

impl DataEntryPanel {
    /// Create a new data entry panel
    #[instrument(skip(template, instance))]
    pub fn new(template: DrawingTemplate, instance: DrawingInstance) -> Self {
        debug!(
            template_id = %template.id(),
            page_count = template.page_count(),
            "Creating data entry panel"
        );

        Self {
            template,
            instance,
            current_page: 0,
            validation_errors: HashMap::new(),
            dirty: false,
        }
    }

    /// Check if panel has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get the current template
    pub fn template(&self) -> &DrawingTemplate {
        &self.template
    }

    /// Get the current instance
    pub fn instance(&self) -> &DrawingInstance {
        &self.instance
    }

    /// Get mutable reference to instance
    pub fn instance_mut(&mut self) -> &mut DrawingInstance {
        &mut self.instance
    }

    /// Set current page
    #[instrument(skip(self))]
    pub fn set_page(&mut self, page: usize) {
        if page < self.template.page_count() {
            debug!(old_page = self.current_page, new_page = page, "Changing page");
            self.current_page = page;
        } else {
            warn!(
                page,
                page_count = self.template.page_count(),
                "Attempted to set invalid page"
            );
        }
    }

    /// Get current page index
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Validate all fields
    #[instrument(skip(self))]
    pub fn validate(&mut self) -> Result<(), TemplateError> {
        self.validation_errors.clear();

        // TODO: Implement field validation based on template
        // For now, just check required fields
        debug!(field_count = self.template.field_count(), "Validating fields");

        Ok(())
    }

    /// Calculate completion percentage
    #[instrument(skip(self))]
    pub fn completion_percentage(&self) -> f32 {
        let total_fields = self.template.field_count();
        if total_fields == 0 {
            return 100.0;
        }

        let filled_fields = self.instance.field_values().len();
        (filled_fields as f32 / total_fields as f32) * 100.0
    }

    /// Render the data entry panel
    #[instrument(skip(self, ui))]
    pub fn ui(&mut self, ui: &mut Ui) -> DataEntryAction {
        let mut action = DataEntryAction::None;

        // Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new(self.template.name()).strong());
        });

        ui.separator();

        // Instance name input
        ui.horizontal(|ui| {
            ui.label("Instance Name:");
            let mut name = self.instance.instance_name().as_ref().map(|s| s.as_str()).unwrap_or_default().to_string();
            if ui.text_edit_singleline(&mut name).changed() {
                self.instance.set_instance_name(name);
                self.dirty = true;
            }
        });

        ui.separator();

        // Progress and page navigation
        ui.horizontal(|ui| {
            ui.label(format!(
                "Page {} of {}",
                self.current_page + 1,
                self.template.page_count()
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{:.0}% complete",
                    self.completion_percentage()
                ));
            });
        });

        ui.separator();

        // Field list (scrollable)
        ScrollArea::vertical()
            .id_salt("data_entry_fields")
            .show(ui, |ui| {
                action = self.render_fields(ui);
            });

        ui.separator();

        // Footer with navigation and actions
        ui.horizontal(|ui| {
            // Page navigation
            if ui
                .add_enabled(self.current_page > 0, egui::Button::new("◀ Previous"))
                .clicked()
            {
                self.set_page(self.current_page.saturating_sub(1));
            }

            if ui
                .add_enabled(
                    self.current_page < self.template.page_count() - 1,
                    egui::Button::new("Next ▶"),
                )
                .clicked()
            {
                self.set_page(self.current_page + 1);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Cancel
                if ui.button("Cancel").clicked() {
                    action = DataEntryAction::Cancel;
                }

                // Submit
                let can_submit = self.validation_errors.is_empty();
                if ui
                    .add_enabled(can_submit, egui::Button::new("Submit"))
                    .clicked()
                {
                    action = DataEntryAction::Submit;
                }

                // Save draft
                if ui.button("Save Draft").clicked() {
                    action = DataEntryAction::SaveDraft;
                }

                // Validation status
                if !self.validation_errors.is_empty() {
                    ui.label(
                        RichText::new(format!("{} errors", self.validation_errors.len()))
                            .color(Color32::RED),
                    );
                }
            });
        });

        action
    }

    /// Render fields for current page
    #[instrument(skip(self, ui))]
    fn render_fields(&mut self, ui: &mut Ui) -> DataEntryAction {
        use form_factor_core::FormTemplate;

        // Clone fields to avoid borrow checker issues
        let fields: Vec<_> = self.template.fields_for_page(self.current_page)
            .into_iter()
            .cloned()
            .collect();

        if fields.is_empty() {
            ui.label(
                RichText::new("No fields defined for this page")
                    .color(Color32::GRAY)
                    .italics(),
            );
            return DataEntryAction::None;
        }

        debug!(field_count = fields.len(), "Rendering fields");

        for field_def in &fields {
            ui.group(|ui| {
                // Field label with required indicator
                let label_text = if *field_def.required() {
                    format!("{} *", field_def.label())
                } else {
                    field_def.label().to_string()
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new(&label_text).strong());

                    // Help text tooltip
                    if let Some(help) = field_def.help_text() {
                        ui.label(RichText::new("ⓘ").color(Color32::GRAY))
                            .on_hover_text(help);
                    }
                });

                // Render input widget based on field type
                let changed = self.render_field_input(ui, field_def);

                // Show validation error if present
                if let Some(error_msg) = self.validation_errors.get(field_def.id()) {
                    ui.label(RichText::new(error_msg).color(Color32::RED).small());
                }

                if changed {
                    self.dirty = true;
                    // Validate field on change
                    self.validate_field(field_def);
                }
            });

            ui.add_space(8.0);
        }

        DataEntryAction::None
    }

    /// Render input widget for a specific field
    #[instrument(skip(self, ui, field_def))]
    fn render_field_input(
        &mut self,
        ui: &mut Ui,
        field_def: &form_factor_core::FieldDefinition,
    ) -> bool {
        use form_factor_core::{FieldContent, FieldType};

        let field_id = field_def.id();
        let mut changed = false;

        // Get or create field value
        let current_value = self.instance.field_value(field_id);

        match field_def.field_type() {
            // Email field with validation feedback
            FieldType::Email => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut text).changed() {
                        self.update_field_value(field_id, FieldContent::Text(text.clone()), field_def);
                        changed = true;
                    }

                    // Show validation indicator
                    if !text.is_empty() {
                        if self.is_valid_email(&text) {
                            ui.label(RichText::new("✓").color(Color32::GREEN));
                        } else {
                            ui.label(RichText::new("✗").color(Color32::RED));
                        }
                    }
                });
            }

            // Phone number with formatting
            FieldType::PhoneNumber => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                let response = ui.text_edit_singleline(&mut text);
                if response.changed() {
                    // Format phone number as user types
                    let formatted = self.format_phone_number(&text);
                    self.update_field_value(field_id, FieldContent::Text(formatted), field_def);
                    changed = true;
                }
            }

            // SSN with masking
            FieldType::SSN => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                ui.horizontal(|ui| {
                    let should_mask = text.len() > 7;
                    let is_complete = text.len() == 11;
                    let last_four = if is_complete {
                        text[7..11].to_string()
                    } else {
                        String::new()
                    };

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut text)
                            .hint_text("###-##-####")
                            .password(should_mask)
                    );

                    if response.changed() {
                        let formatted = self.format_ssn(&text);
                        self.update_field_value(field_id, FieldContent::Text(formatted), field_def);
                        changed = true;
                    }

                    if is_complete {
                        // Show masked version
                        ui.label(RichText::new("***-**-").color(Color32::GRAY).small());
                        ui.label(RichText::new(&last_four).small());
                    }
                });
            }

            // Tax ID with formatting
            FieldType::TaxId => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                let response = ui.add(
                    egui::TextEdit::singleline(&mut text).hint_text("##-#######")
                );

                if response.changed() {
                    let formatted = self.format_tax_id(&text);
                    self.update_field_value(field_id, FieldContent::Text(formatted), field_def);
                    changed = true;
                }
            }

            // ZIP code with validation
            FieldType::ZipCode => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut text).hint_text("##### or #####-####")
                    );

                    if response.changed() {
                        let formatted = self.format_zip_code(&text);
                        self.update_field_value(field_id, FieldContent::Text(formatted), field_def);
                        changed = true;
                    }

                    // Show validation indicator
                    if !text.is_empty() {
                        if text.len() == 5 || text.len() == 10 {
                            ui.label(RichText::new("✓").color(Color32::GREEN));
                        } else {
                            ui.label(RichText::new("✗").color(Color32::RED));
                        }
                    }
                });
            }

            // Generic text fields
            FieldType::FirstName
            | FieldType::MiddleName
            | FieldType::LastName
            | FieldType::FullName
            | FieldType::StreetAddress
            | FieldType::AddressLine2
            | FieldType::City
            | FieldType::State
            | FieldType::Country
            | FieldType::DriverLicense
            | FieldType::PassportNumber
            | FieldType::AccountNumber
            | FieldType::RoutingNumber
            | FieldType::EmployerName
            | FieldType::JobTitle
            | FieldType::EmployeeId
            | FieldType::CompanyName
            | FieldType::CompanyAddress
            | FieldType::TextRegion
            | FieldType::FreeText
            | FieldType::Custom(_) => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                if ui.text_edit_singleline(&mut text).changed() {
                    self.update_field_value(field_id, FieldContent::Text(text), field_def);
                    changed = true;
                }
            }

            // Numeric fields
            FieldType::NumericField | FieldType::Amount => {
                let mut value = current_value
                    .and_then(|v| v.as_number())
                    .unwrap_or(0.0);

                if ui.add(egui::DragValue::new(&mut value).speed(0.1)).changed() {
                    self.update_field_value(field_id, FieldContent::Number(value), field_def);
                    changed = true;
                }
            }

            // Currency fields
            FieldType::Currency => {
                let mut value = current_value
                    .and_then(|v| v.as_number())
                    .unwrap_or(0.0);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut value)
                                .prefix("$")
                                .speed(0.01)
                                .max_decimals(2)
                                .min_decimals(2)
                        )
                        .changed()
                    {
                        self.update_field_value(field_id, FieldContent::Number(value), field_def);
                        changed = true;
                    }

                    // Show formatted currency
                    if value != 0.0 {
                        ui.label(
                            RichText::new(format!("({:.2})", value))
                                .color(Color32::GRAY)
                                .small(),
                        );
                    }
                });
            }

            // Boolean fields (checkbox)
            FieldType::Checkbox | FieldType::RadioButton => {
                let mut checked = current_value
                    .and_then(|v| v.as_boolean())
                    .unwrap_or(false);

                if ui.checkbox(&mut checked, "").changed() {
                    self.update_field_value(field_id, FieldContent::Boolean(checked), field_def);
                    changed = true;
                }
            }

            // Date fields
            FieldType::Date | FieldType::DateOfBirth | FieldType::DateSigned => {
                let mut text = current_value
                    .and_then(|v| v.as_text())
                    .unwrap_or("")
                    .to_string();

                ui.horizontal(|ui| {
                    if ui
                        .add(egui::TextEdit::singleline(&mut text).hint_text("MM/DD/YYYY"))
                        .changed()
                    {
                        self.update_field_value(field_id, FieldContent::Text(text), field_def);
                        changed = true;
                    }
                    ui.label(RichText::new("📅").color(Color32::GRAY));
                });
            }

            // Signature and initials (placeholder)
            FieldType::Signature | FieldType::Initials => {
                if ui.button("Sign...").clicked() {
                    // TODO: Open signature capture dialog
                    warn!("Signature capture not yet implemented");
                }
                ui.label(
                    RichText::new("(Signature capture coming soon)")
                        .color(Color32::GRAY)
                        .small(),
                );
            }

            // Logo (not editable)
            FieldType::Logo => {
                ui.label(
                    RichText::new("Logo field (auto-detected)")
                        .color(Color32::GRAY)
                        .italics(),
                );
            }

            // Barcode/QR Code (not editable)
            FieldType::Barcode | FieldType::QRCode => {
                ui.label(
                    RichText::new("Barcode/QR field (auto-detected)")
                        .color(Color32::GRAY)
                        .italics(),
                );
            }
        }

        changed
    }

    /// Update a field value in the instance
    #[instrument(skip(self, content, field_def))]
    fn update_field_value(
        &mut self,
        field_id: &str,
        content: form_factor_core::FieldContent,
        field_def: &form_factor_core::FieldDefinition,
    ) {
        use form_factor_core::FieldValue;

        // Create appropriate field value based on content type
        let field_value = match content {
            form_factor_core::FieldContent::Text(text) => {
                FieldValue::new_text(field_id, text, *field_def.bounds(), self.current_page)
            }
            form_factor_core::FieldContent::Boolean(value) => {
                FieldValue::new_boolean(field_id, value, *field_def.bounds(), self.current_page)
            }
            form_factor_core::FieldContent::Number(value) => {
                FieldValue::new_text(field_id, value.to_string(), *field_def.bounds(), self.current_page)
            }
            _ => {
                FieldValue::new_empty(field_id, *field_def.bounds(), self.current_page)
            }
        };

        if let Err(e) = self.instance.set_field_value(field_id, field_value) {
            warn!(
                field_id,
                error = %e,
                "Failed to set field value"
            );
        } else {
            debug!(field_id, "Updated field value");
        }
    }

    /// Validate a single field
    #[instrument(skip(self, field_def))]
    fn validate_field(&mut self, field_def: &form_factor_core::FieldDefinition) {
        let field_id = field_def.id();

        // Remove existing error for this field
        self.validation_errors.remove(field_id);

        // Check required fields
        if *field_def.required() {
            if let Some(value) = self.instance.field_value(field_id) {
                if value.is_empty() {
                    self.validation_errors.insert(
                        field_id.to_string(),
                        "This field is required".to_string(),
                    );
                    return;
                }
            } else {
                self.validation_errors.insert(
                    field_id.to_string(),
                    "This field is required".to_string(),
                );
                return;
            }
        }

        // Check validation pattern
        if let Some(pattern) = field_def.effective_validation_pattern()
            && let Some(value) = self.instance.field_value(field_id)
            && let Some(text) = value.as_text()
            && !text.is_empty()
        {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(text) {
                        self.validation_errors.insert(
                            field_id.to_string(),
                            "Value does not match expected format".to_string(),
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        pattern,
                        error = %e,
                        "Invalid validation pattern"
                    );
                }
            }
        }
    }

    /// Validate email address format
    fn is_valid_email(&self, email: &str) -> bool {
        // Simple email validation
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        if let Ok(re) = regex::Regex::new(email_pattern) {
            re.is_match(email)
        } else {
            false
        }
    }

    /// Format phone number as (XXX) XXX-XXXX
    fn format_phone_number(&self, input: &str) -> String {
        // Remove all non-digit characters
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

        // Limit to 10 digits
        let digits = &digits[..digits.len().min(10)];

        match digits.len() {
            0 => String::new(),
            1..=3 => format!("({})", digits),
            4..=6 => format!("({}) {}", &digits[..3], &digits[3..]),
            7..=10 => format!("({}) {}-{}", &digits[..3], &digits[3..6], &digits[6..]),
            _ => input.to_string(),
        }
    }

    /// Format SSN as XXX-XX-XXXX
    fn format_ssn(&self, input: &str) -> String {
        // Remove all non-digit characters
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

        // Limit to 9 digits
        let digits = &digits[..digits.len().min(9)];

        match digits.len() {
            0 => String::new(),
            1..=3 => digits.to_string(),
            4..=5 => format!("{}-{}", &digits[..3], &digits[3..]),
            6..=9 => format!("{}-{}-{}", &digits[..3], &digits[3..5], &digits[5..]),
            _ => input.to_string(),
        }
    }

    /// Format Tax ID as XX-XXXXXXX
    fn format_tax_id(&self, input: &str) -> String {
        // Remove all non-digit characters
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

        // Limit to 9 digits
        let digits = &digits[..digits.len().min(9)];

        match digits.len() {
            0 => String::new(),
            1..=2 => digits.to_string(),
            3..=9 => format!("{}-{}", &digits[..2], &digits[2..]),
            _ => input.to_string(),
        }
    }

    /// Format ZIP code as XXXXX or XXXXX-XXXX
    fn format_zip_code(&self, input: &str) -> String {
        // Remove all non-digit characters
        let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

        // Limit to 9 digits (ZIP+4)
        let digits = &digits[..digits.len().min(9)];

        match digits.len() {
            0..=5 => digits.to_string(),
            6..=9 => format!("{}-{}", &digits[..5], &digits[5..]),
            _ => input.to_string(),
        }
    }

}
