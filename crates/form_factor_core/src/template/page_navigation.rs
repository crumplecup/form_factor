use derive_getters::Getters;

/// Page navigation state for multi-page templates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct PageNavigation {
    /// Current page index (0-based).
    current_page: usize,
    
    /// Total number of pages.
    total_pages: usize,
}

impl PageNavigation {
    /// Creates a new page navigation with the specified total pages.
    #[must_use]
    pub fn new(total_pages: usize) -> Self {
        Self {
            current_page: 0,
            total_pages: total_pages.max(1),
        }
    }
    
    /// Navigates to the next page if available.
    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages {
            self.current_page += 1;
        }
    }
    
    /// Navigates to the previous page if available.
    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }
    
    /// Navigates to a specific page index.
    pub fn goto_page(&mut self, page: usize) {
        if page < self.total_pages {
            self.current_page = page;
        }
    }
    
    /// Adds a new page, increasing the total count.
    pub fn add_page(&mut self) {
        self.total_pages += 1;
    }
    
    /// Removes the current page if more than one page exists.
    pub fn remove_current_page(&mut self) {
        if self.total_pages > 1 {
            self.total_pages -= 1;
            if self.current_page >= self.total_pages {
                self.current_page = self.total_pages - 1;
            }
        }
    }
    
    /// Checks if there is a next page available.
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.current_page + 1 < self.total_pages
    }
    
    /// Checks if there is a previous page available.
    #[must_use]
    pub fn has_previous(&self) -> bool {
        self.current_page > 0
    }
}
