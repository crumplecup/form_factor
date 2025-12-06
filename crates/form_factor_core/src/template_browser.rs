//! Template browser for managing and organizing form templates.

use crate::{TemplateBrowserError, TemplateBrowserErrorKind, TemplateBrowserResult};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, instrument, warn};

/// UI state for browsing and managing templates.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(setter(into))]
pub struct TemplateBrowser {
    /// List of available templates with metadata.
    #[builder(default)]
    templates: Vec<TemplateEntry>,

    /// Currently selected template index.
    #[builder(default)]
    selected_index: Option<usize>,

    /// Search/filter text.
    #[builder(default)]
    filter_text: String,

    /// Sort order for template list.
    #[builder(default = "SortOrder::NameAscending")]
    sort_order: SortOrder,

    /// Whether the browser panel is expanded.
    #[builder(default = "true")]
    expanded: bool,
}

impl TemplateBrowser {
    /// Creates a new template browser.
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating new template browser");
        TemplateBrowserBuilder::default()
            .build()
            .expect("TemplateBrowser builder should not fail with defaults")
    }

    /// Adds a template to the browser.
    #[instrument(skip(self, entry), fields(template_id = %entry.template_id))]
    pub fn add_template(&mut self, entry: TemplateEntry) {
        debug!("Adding template to browser");
        self.templates.push(entry);
        self.sort_templates();
    }

    /// Removes a template by index.
    #[instrument(skip(self), fields(index, template_count = self.templates.len()))]
    pub fn remove_template(&mut self, index: usize) -> TemplateBrowserResult<TemplateEntry> {
        if index >= self.templates.len() {
            warn!(index, max = self.templates.len() - 1, "Template index out of bounds");
            return Err(TemplateBrowserError::new(
                TemplateBrowserErrorKind::IndexOutOfBounds {
                    index,
                    max: self.templates.len().saturating_sub(1),
                },
            ));
        }

        let entry = self.templates.remove(index);
        debug!(template_id = %entry.template_id, "Removed template");

        if self.selected_index == Some(index) {
            self.selected_index = None;
        } else if let Some(selected) = self.selected_index
            && selected > index
        {
            self.selected_index = Some(selected - 1);
        }

        Ok(entry)
    }

    /// Gets the currently selected template.
    #[instrument(skip(self))]
    pub fn selected_template(&self) -> Option<&TemplateEntry> {
        self.selected_index
            .and_then(|idx| self.templates.get(idx))
    }

    /// Sets the selected template index.
    #[instrument(skip(self))]
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        debug!(?index, "Setting selected template index");
        self.selected_index = index;
    }

    /// Updates the filter text.
    #[instrument(skip(self), fields(text_len = text.len()))]
    pub fn set_filter_text(&mut self, text: String) {
        debug!("Updating filter text");
        self.filter_text = text;
    }

    /// Gets filtered templates based on current filter text.
    #[instrument(skip(self), fields(filter = %self.filter_text, total = self.templates.len()))]
    pub fn filtered_templates(&self) -> Vec<(usize, &TemplateEntry)> {
        if self.filter_text.is_empty() {
            self.templates.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.templates
                .iter()
                .enumerate()
                .filter(|(_, entry)| {
                    entry.metadata.name().to_lowercase().contains(&filter_lower)
                        || entry
                            .metadata
                            .description()
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&filter_lower))
                            .unwrap_or(false)
                })
                .collect()
        }
    }

    /// Sets the sort order and re-sorts templates.
    #[instrument(skip(self), fields(order = ?order))]
    pub fn set_sort_order(&mut self, order: SortOrder) {
        debug!("Setting sort order and resorting templates");
        self.sort_order = order;
        self.sort_templates();
    }

    /// Sorts templates according to current sort order.
    #[instrument(skip(self), fields(order = ?self.sort_order, count = self.templates.len()))]
    fn sort_templates(&mut self) {
        debug!("Sorting templates");

        match self.sort_order {
            SortOrder::NameAscending => {
                self.templates
                    .sort_by(|a, b| a.metadata.name().cmp(b.metadata.name()));
            }
            SortOrder::NameDescending => {
                self.templates
                    .sort_by(|a, b| b.metadata.name().cmp(a.metadata.name()));
            }
            SortOrder::DateCreatedNewest => {
                self.templates
                    .sort_by(|a, b| b.metadata.created_at().cmp(a.metadata.created_at()));
            }
            SortOrder::DateCreatedOldest => {
                self.templates
                    .sort_by(|a, b| a.metadata.created_at().cmp(b.metadata.created_at()));
            }
            SortOrder::DateModifiedNewest => {
                self.templates
                    .sort_by(|a, b| b.metadata.modified_at().cmp(a.metadata.modified_at()));
            }
            SortOrder::DateModifiedOldest => {
                self.templates
                    .sort_by(|a, b| a.metadata.modified_at().cmp(b.metadata.modified_at()));
            }
        }
    }

    /// Toggles the expanded state.
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Sets the expanded state.
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }
}

impl Default for TemplateBrowser {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry in the template browser list.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(setter(into))]
pub struct TemplateEntry {
    /// Template ID
    template_id: String,

    /// Template metadata.
    metadata: TemplateMetadata,

    /// File path where template is stored.
    #[builder(default)]
    file_path: Option<PathBuf>,

    /// Thumbnail image data (if available).
    #[builder(default)]
    thumbnail: Option<Vec<u8>>,
}

impl TemplateEntry {
    /// Creates a new template entry.
    #[instrument(skip(metadata), fields(template_id = %template_id))]
    pub fn new(template_id: String, metadata: TemplateMetadata) -> Self {
        debug!("Creating template entry");
        TemplateEntryBuilder::default()
            .template_id(template_id)
            .metadata(metadata)
            .build()
            .expect("TemplateEntry builder should not fail")
    }

    /// Sets the file path.
    pub fn set_file_path(&mut self, path: Option<PathBuf>) {
        self.file_path = path;
    }

    /// Sets the thumbnail.
    pub fn set_thumbnail(&mut self, thumbnail: Option<Vec<u8>>) {
        self.thumbnail = thumbnail;
    }
}

/// Metadata about a template for browsing and display.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Builder)]
#[builder(setter(into))]
pub struct TemplateMetadata {
    /// Template name for display.
    name: String,

    /// Template description.
    #[builder(default)]
    description: Option<String>,

    /// Template version.
    #[builder(default = "String::from(\"1.0.0\")")]
    version: String,

    /// Number of pages.
    #[builder(default = "1")]
    page_count: usize,

    /// Number of fields.
    #[builder(default = "0")]
    field_count: usize,

    /// When the template was created.
    #[builder(default = "Utc::now()")]
    created_at: DateTime<Utc>,

    /// When the template was last modified.
    #[builder(default = "Utc::now()")]
    modified_at: DateTime<Utc>,

    /// Tags for categorization.
    #[builder(default)]
    tags: Vec<String>,
}

impl TemplateMetadata {
    /// Creates a new template metadata.
    #[instrument(fields(name = %name))]
    pub fn new(name: String) -> Self {
        debug!("Creating template metadata");
        TemplateMetadataBuilder::default()
            .name(name)
            .build()
            .expect("TemplateMetadata builder should not fail")
    }

    /// Sets the modified timestamp to now.
    #[instrument(skip(self))]
    pub fn touch(&mut self) {
        debug!("Updating modified timestamp");
        self.modified_at = Utc::now();
    }

    /// Adds a tag.
    #[instrument(skip(self), fields(tag = %tag))]
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            debug!("Adding tag");
            self.tags.push(tag);
        }
    }

    /// Removes a tag.
    #[instrument(skip(self), fields(tag = %tag))]
    pub fn remove_tag(&mut self, tag: &str) {
        debug!("Removing tag");
        self.tags.retain(|t| t != tag);
    }
}

/// Sort order for template list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortOrder {
    /// Sort by name A-Z.
    NameAscending,
    /// Sort by name Z-A.
    NameDescending,
    /// Sort by creation date, newest first.
    DateCreatedNewest,
    /// Sort by creation date, oldest first.
    DateCreatedOldest,
    /// Sort by modification date, newest first.
    DateModifiedNewest,
    /// Sort by modification date, oldest first.
    DateModifiedOldest,
}

impl SortOrder {
    /// Returns all sort order variants.
    pub fn all() -> &'static [SortOrder] {
        &[
            SortOrder::NameAscending,
            SortOrder::NameDescending,
            SortOrder::DateCreatedNewest,
            SortOrder::DateCreatedOldest,
            SortOrder::DateModifiedNewest,
            SortOrder::DateModifiedOldest,
        ]
    }

    /// Returns a display name for the sort order.
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::NameAscending => "Name (A-Z)",
            SortOrder::NameDescending => "Name (Z-A)",
            SortOrder::DateCreatedNewest => "Created (Newest)",
            SortOrder::DateCreatedOldest => "Created (Oldest)",
            SortOrder::DateModifiedNewest => "Modified (Newest)",
            SortOrder::DateModifiedOldest => "Modified (Oldest)",
        }
    }
}
