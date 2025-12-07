//! Field type selector UI component.

use egui::{Color32, Ui};
use form_factor_core::FieldType;
use tracing::{debug, instrument};

/// Field type selector widget.
#[derive(Debug, Clone)]
pub struct FieldTypeSelector {
    /// Current search filter text.
    search_text: String,
    /// Currently selected field type.
    selected: Option<FieldType>,
    /// Whether to show all categories expanded.
    show_all: bool,
}

impl Default for FieldTypeSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldTypeSelector {
    /// Creates a new field type selector.
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating field type selector");
        Self {
            search_text: String::new(),
            selected: None,
            show_all: false,
        }
    }

    /// Sets the initially selected field type.
    #[instrument(skip(self), fields(field_type = ?field_type))]
    pub fn with_selected(mut self, field_type: FieldType) -> Self {
        debug!("Setting initial selection");
        self.selected = Some(field_type);
        self
    }

    /// Gets the currently selected field type.
    pub fn selected(&self) -> Option<&FieldType> {
        self.selected.as_ref()
    }

    /// Shows the selector UI.
    ///
    /// Returns true if the selection changed.
    #[instrument(skip(self, ui), fields(has_selection = self.selected.is_some()))]
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        // Search box
        ui.horizontal(|ui| {
            ui.label("Search:");
            if ui.text_edit_singleline(&mut self.search_text).changed() {
                self.show_all = !self.search_text.is_empty();
            }
        });

        ui.separator();

        // Scrollable area for field types
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                let categories = Self::field_type_categories();
                let search_lower = self.search_text.to_lowercase();

                for (category_name, field_types) in categories {
                    // Filter field types based on search
                    let filtered: Vec<_> = field_types
                        .iter()
                        .filter(|ft| {
                            if search_lower.is_empty() {
                                true
                            } else {
                                ft.display_name().to_lowercase().contains(&search_lower)
                                    || category_name.to_lowercase().contains(&search_lower)
                            }
                        })
                        .collect();

                    if filtered.is_empty() {
                        continue;
                    }

                    // Category header - auto-open when searching
                    let id = ui.make_persistent_id(format!("category_{}", category_name));
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        self.show_all,
                    )
                    .show_header(ui, |ui| {
                        ui.label(category_name);
                    })
                    .body(|ui| {
                        for field_type in filtered {
                            if self.show_field_type_button(ui, field_type) {
                                debug!(field_type = ?field_type, "Field type selected");
                                self.selected = Some(field_type.clone());
                                changed = true;
                            }
                        }
                    });
                }
            });

        // Display current selection
        if let Some(ref selected) = self.selected {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Selected:");
                ui.label(egui::RichText::new(selected.display_name()).strong());
            });

            // Show validation pattern if available
            if let Some(pattern) = selected.validation_pattern() {
                ui.horizontal(|ui| {
                    ui.label("Pattern:");
                    ui.label(egui::RichText::new(pattern).code());
                });
            }
        }

        changed
    }

    /// Shows a single field type as a selectable button.
    fn show_field_type_button(&self, ui: &mut Ui, field_type: &FieldType) -> bool {
        let is_selected = self.selected.as_ref() == Some(field_type);

        let button = egui::Button::new(field_type.display_name())
            .min_size(egui::vec2(ui.available_width(), 0.0))
            .fill(if is_selected {
                Color32::from_rgb(100, 150, 200)
            } else {
                Color32::TRANSPARENT
            });

        ui.add(button).clicked()
    }

    /// Gets field types organized by category.
    fn field_type_categories() -> Vec<(&'static str, Vec<FieldType>)> {
        vec![
            (
                "Personal Information",
                vec![
                    FieldType::FirstName,
                    FieldType::MiddleName,
                    FieldType::LastName,
                    FieldType::FullName,
                    FieldType::Email,
                    FieldType::PhoneNumber,
                    FieldType::DateOfBirth,
                ],
            ),
            (
                "Address",
                vec![
                    FieldType::StreetAddress,
                    FieldType::AddressLine2,
                    FieldType::City,
                    FieldType::State,
                    FieldType::ZipCode,
                    FieldType::Country,
                ],
            ),
            (
                "Identification",
                vec![
                    FieldType::SSN,
                    FieldType::TaxId,
                    FieldType::DriverLicense,
                    FieldType::PassportNumber,
                ],
            ),
            ("Dates", vec![FieldType::Date, FieldType::DateSigned]),
            (
                "Financial",
                vec![
                    FieldType::AccountNumber,
                    FieldType::RoutingNumber,
                    FieldType::Currency,
                    FieldType::Amount,
                ],
            ),
            (
                "Employment",
                vec![
                    FieldType::EmployerName,
                    FieldType::JobTitle,
                    FieldType::EmployeeId,
                ],
            ),
            (
                "Company/Organization",
                vec![
                    FieldType::CompanyName,
                    FieldType::CompanyAddress,
                    FieldType::Logo,
                ],
            ),
            (
                "Form Controls",
                vec![
                    FieldType::Checkbox,
                    FieldType::RadioButton,
                    FieldType::Signature,
                    FieldType::Initials,
                ],
            ),
            (
                "Generic Fields",
                vec![
                    FieldType::TextRegion,
                    FieldType::NumericField,
                    FieldType::FreeText,
                    FieldType::Barcode,
                    FieldType::QRCode,
                ],
            ),
        ]
    }
}
