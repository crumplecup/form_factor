//! Form template definitions and traits
//!
//! This module provides the core abstractions for defining form templates.
//! Templates describe the structure and expected fields of a form type,
//! independent of any specific instance or filled data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for form template definitions
///
/// A form template defines the structure and expected fields of a form type.
/// Templates are stored in a registry and can be versioned for evolution.
/// Implementations must be Send + Sync to support concurrent access.
pub trait FormTemplate: Send + Sync {
    /// Unique identifier for this template
    ///
    /// Examples: "irs_w2_2024", "loan_application_v3", "invoice_standard"
    fn id(&self) -> &str;

    /// Human-readable name for display in UI
    ///
    /// Examples: "IRS Form W-2 (2024)", "Loan Application", "Standard Invoice"
    fn name(&self) -> &str;

    /// Template version for evolution tracking
    ///
    /// Use semantic versioning: "1.0.0", "2.1.0", etc.
    fn version(&self) -> &str;

    /// Optional description of this template
    fn description(&self) -> Option<&str> {
        None
    }

    /// Number of pages in this form template
    fn page_count(&self) -> usize;

    /// Get all field definitions across all pages
    fn fields(&self) -> Vec<&FieldDefinition>;

    /// Get fields for a specific page (0-indexed)
    ///
    /// Returns empty vector if page index is invalid.
    fn fields_for_page(&self, page_index: usize) -> Vec<&FieldDefinition>;

    /// Get a specific field definition by ID
    fn field_by_id(&self, field_id: &str) -> Option<&FieldDefinition>;

    /// Validate that an instance conforms to this template
    ///
    /// Checks:
    /// - All required fields are present
    /// - Field values match expected types
    /// - Validation patterns pass
    /// - Page counts match
    fn validate_instance(&self, instance: &dyn crate::FormInstance) -> crate::ValidationResult;

    /// Get metadata key-value pairs for this template
    fn metadata(&self) -> &HashMap<String, String>;

    /// Serialize to JSON for storage
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>>;

    /// Get expected dimensions for a page (width, height in pixels)
    ///
    /// Returns None if page index is invalid or dimensions not specified.
    fn page_dimensions(&self, page_index: usize) -> Option<(u32, u32)>;
}

/// Semantic field types for form fields
///
/// These encode business meaning and enable intelligent validation,
/// formatting, and data extraction. Each type may have associated
/// validation patterns and formatting rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    // Personal Information
    /// First name only
    FirstName,
    /// Middle name or initial
    MiddleName,
    /// Last name/surname only
    LastName,
    /// Full legal name (first + middle + last)
    FullName,
    /// Email address
    Email,
    /// Phone number (various formats supported)
    PhoneNumber,

    // Address Fields
    /// Street address line 1
    StreetAddress,
    /// Apartment, suite, unit number
    AddressLine2,
    /// City name
    City,
    /// State or province (2-letter code or full name)
    State,
    /// ZIP or postal code
    ZipCode,
    /// Country name or code
    Country,

    // Identification Numbers
    /// Social Security Number (XXX-XX-XXXX)
    SSN,
    /// Tax ID / EIN
    TaxId,
    /// Driver's license number
    DriverLicense,
    /// Passport number
    PassportNumber,

    // Dates
    /// Date of birth
    DateOfBirth,
    /// Generic date field
    Date,
    /// Date document was signed
    DateSigned,

    // Financial Fields
    /// Bank account number
    AccountNumber,
    /// Bank routing number
    RoutingNumber,
    /// Currency amount with symbol
    Currency,
    /// Numeric amount
    Amount,

    // Employment
    /// Employer or company name
    EmployerName,
    /// Job title or position
    JobTitle,
    /// Employee ID number
    EmployeeId,

    // Form Controls
    /// Checkbox (boolean value)
    Checkbox,
    /// Radio button (one of many selection)
    RadioButton,
    /// Signature area
    Signature,
    /// Initials area
    Initials,

    // Company/Organization
    /// Company or business name
    CompanyName,
    /// Company address
    CompanyAddress,
    /// Logo image region
    Logo,

    // Generic Fields
    /// Generic text region (no specific validation)
    TextRegion,
    /// Numeric field (integer or decimal)
    NumericField,
    /// Free-form text area
    FreeText,
    /// Barcode region
    Barcode,
    /// QR code region
    QRCode,

    // Extensibility
    /// Custom field type with user-defined name
    ///
    /// Use this for application-specific field types not covered above.
    /// Example: Custom("claim_number".into())
    Custom(String),
}

impl FieldType {
    /// Get default validation pattern (regex) for this field type
    ///
    /// Returns None for types that don't have standard validation patterns.
    pub fn validation_pattern(&self) -> Option<&'static str> {
        match self {
            FieldType::Email => Some(r"^[^\s@]+@[^\s@]+\.[^\s@]+$"),
            FieldType::SSN => Some(r"^\d{3}-\d{2}-\d{4}$"),
            FieldType::PhoneNumber => Some(r"^\+?1?\s*\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"),
            FieldType::ZipCode => Some(r"^\d{5}(-\d{4})?$"),
            FieldType::Date | FieldType::DateOfBirth | FieldType::DateSigned => {
                Some(r"^\d{1,2}/\d{1,2}/\d{4}$")
            }
            FieldType::State => Some(r"^[A-Z]{2}$"),
            FieldType::TaxId => Some(r"^\d{2}-\d{7}$"),
            FieldType::RoutingNumber => Some(r"^\d{9}$"),
            _ => None,
        }
    }

    /// Check if this field type expects text content
    ///
    /// Returns false for binary/visual field types like signatures, logos, barcodes.
    pub fn expects_text(&self) -> bool {
        !matches!(
            self,
            FieldType::Checkbox
                | FieldType::RadioButton
                | FieldType::Signature
                | FieldType::Initials
                | FieldType::Logo
                | FieldType::Barcode
                | FieldType::QRCode
        )
    }

    /// Check if this field type requires OCR for text extraction
    pub fn requires_ocr(&self) -> bool {
        self.expects_text()
    }

    /// Get a human-readable display name for this field type
    pub fn display_name(&self) -> &str {
        match self {
            FieldType::FirstName => "First Name",
            FieldType::MiddleName => "Middle Name",
            FieldType::LastName => "Last Name",
            FieldType::FullName => "Full Name",
            FieldType::Email => "Email Address",
            FieldType::PhoneNumber => "Phone Number",
            FieldType::StreetAddress => "Street Address",
            FieldType::AddressLine2 => "Address Line 2",
            FieldType::City => "City",
            FieldType::State => "State",
            FieldType::ZipCode => "ZIP Code",
            FieldType::Country => "Country",
            FieldType::SSN => "Social Security Number",
            FieldType::TaxId => "Tax ID",
            FieldType::DriverLicense => "Driver's License",
            FieldType::PassportNumber => "Passport Number",
            FieldType::DateOfBirth => "Date of Birth",
            FieldType::Date => "Date",
            FieldType::DateSigned => "Date Signed",
            FieldType::AccountNumber => "Account Number",
            FieldType::RoutingNumber => "Routing Number",
            FieldType::Currency => "Currency",
            FieldType::Amount => "Amount",
            FieldType::EmployerName => "Employer Name",
            FieldType::JobTitle => "Job Title",
            FieldType::EmployeeId => "Employee ID",
            FieldType::Checkbox => "Checkbox",
            FieldType::RadioButton => "Radio Button",
            FieldType::Signature => "Signature",
            FieldType::Initials => "Initials",
            FieldType::CompanyName => "Company Name",
            FieldType::CompanyAddress => "Company Address",
            FieldType::Logo => "Logo",
            FieldType::TextRegion => "Text Region",
            FieldType::NumericField => "Numeric Field",
            FieldType::FreeText => "Free Text",
            FieldType::Barcode => "Barcode",
            FieldType::QRCode => "QR Code",
            FieldType::Custom(name) => name,
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Definition of a field in a template
///
/// Describes a single field's location, type, validation rules, and metadata.
/// Field definitions are immutable once created; use a builder to construct them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_getters::Getters)]
pub struct FieldDefinition {
    /// Unique ID within the template
    ///
    /// Must be unique across all fields in all pages of the template.
    /// Convention: lowercase_with_underscores
    /// Examples: "employee_name", "gross_wages", "signature_date"
    id: String,

    /// Human-readable label for display in UI
    ///
    /// Examples: "Employee Name", "Gross Wages", "Signature Date"
    label: String,

    /// Semantic field type
    field_type: FieldType,

    /// Which page this field appears on (0-indexed)
    page_index: usize,

    /// Expected position and dimensions (in image pixel coordinates)
    ///
    /// This is the template default. Instances may have different actual
    /// positions based on detection or manual placement.
    bounds: FieldBounds,

    /// Whether this field is required for validation
    required: bool,

    /// Optional custom validation pattern (regex)
    ///
    /// If None, uses FieldType::validation_pattern() default.
    /// If Some, overrides the default pattern.
    validation_pattern: Option<String>,

    /// Optional help text for users filling the field
    help_text: Option<String>,

    /// Additional metadata for this field
    ///
    /// Can store application-specific data like:
    /// - "export_column": "employee_ssn"
    /// - "tax_box": "a"
    /// - "ocr_confidence_threshold": "0.8"
    metadata: HashMap<String, String>,
}

impl FieldDefinition {
    /// Create a new field definition builder
    pub fn builder() -> FieldDefinitionBuilder {
        FieldDefinitionBuilder::default()
    }

    /// Get the effective validation pattern for this field
    ///
    /// Returns custom pattern if set, otherwise the field type's default pattern.
    pub fn effective_validation_pattern(&self) -> Option<&str> {
        self.validation_pattern
            .as_deref()
            .or_else(|| self.field_type.validation_pattern())
    }
}

/// Builder for FieldDefinition
///
/// Provides a fluent API for constructing field definitions.
#[derive(Debug, Default)]
pub struct FieldDefinitionBuilder {
    id: Option<String>,
    label: Option<String>,
    field_type: Option<FieldType>,
    page_index: Option<usize>,
    bounds: Option<FieldBounds>,
    required: bool,
    validation_pattern: Option<String>,
    help_text: Option<String>,
    metadata: HashMap<String, String>,
}

impl FieldDefinitionBuilder {
    /// Set the field ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the field label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the field type
    pub fn field_type(mut self, field_type: FieldType) -> Self {
        self.field_type = Some(field_type);
        self
    }

    /// Set the page index
    pub fn page_index(mut self, page_index: usize) -> Self {
        self.page_index = Some(page_index);
        self
    }

    /// Set the field bounds
    pub fn bounds(mut self, bounds: FieldBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Set whether the field is required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set a custom validation pattern
    pub fn validation_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.validation_pattern = Some(pattern.into());
        self
    }

    /// Set help text
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Add a metadata entry
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the FieldDefinition
    ///
    /// Returns an error if required fields are missing.
    pub fn build(self) -> Result<FieldDefinition, String> {
        Ok(FieldDefinition {
            id: self.id.ok_or("id is required")?,
            label: self.label.ok_or("label is required")?,
            field_type: self.field_type.ok_or("field_type is required")?,
            page_index: self.page_index.ok_or("page_index is required")?,
            bounds: self.bounds.ok_or("bounds is required")?,
            required: self.required,
            validation_pattern: self.validation_pattern,
            help_text: self.help_text,
            metadata: self.metadata,
        })
    }
}

/// Bounding box for a field location
///
/// Coordinates are in image pixel space (top-left origin).
/// All values must be non-negative finite numbers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
pub struct FieldBounds {
    /// X coordinate of top-left corner (image pixel space)
    x: f32,
    /// Y coordinate of top-left corner (image pixel space)
    y: f32,
    /// Width in pixels
    width: f32,
    /// Height in pixels
    height: f32,
}

impl FieldBounds {
    /// Create new field bounds
    ///
    /// # Panics
    ///
    /// Panics if any value is negative, infinite, or NaN.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        assert!(
            x.is_finite() && x >= 0.0,
            "x must be non-negative and finite"
        );
        assert!(
            y.is_finite() && y >= 0.0,
            "y must be non-negative and finite"
        );
        assert!(
            width.is_finite() && width >= 0.0,
            "width must be non-negative and finite"
        );
        assert!(
            height.is_finite() && height >= 0.0,
            "height must be non-negative and finite"
        );

        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to a pair of opposite corners for shape creation
    ///
    /// Returns (top_left, bottom_right) corners.
    pub fn to_corners(&self) -> (egui::Pos2, egui::Pos2) {
        let top_left = egui::Pos2::new(self.x, self.y);
        let bottom_right = egui::Pos2::new(self.x + self.width, self.y + self.height);
        (top_left, bottom_right)
    }

    /// Create from two opposite corners
    ///
    /// Automatically normalizes so top-left is actually top-left.
    pub fn from_corners(start: egui::Pos2, end: egui::Pos2) -> Self {
        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        let width = (end.x - start.x).abs();
        let height = (end.y - start.y).abs();
        Self::new(x, y, width, height)
    }

    /// Get the center point of this bounds
    pub fn center(&self) -> egui::Pos2 {
        egui::Pos2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if a point is inside these bounds
    pub fn contains(&self, point: egui::Pos2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Get the area of this bounds
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}
