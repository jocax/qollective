use serde::{Deserialize, Serialize};

/// Layout mode based on realistic terminal dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutMode {
    /// 4K mode: 480×120+ - Maximum desktop experience
    FourK,
    /// Full HD mode: 240×60+ - Full-featured desktop TUI
    FullHD,
    /// Modern mode: 120×30+ - Standard modern terminal
    Modern,
    /// Classic mode: <120×30 - Traditional 80×24 terminal
    Classic,
}

impl LayoutMode {
    /// Get the display name for this layout mode
    pub fn display_name(&self) -> &'static str {
        match self {
            LayoutMode::FourK => "4K (480×120)",
            LayoutMode::FullHD => "Full HD (240×60)",
            LayoutMode::Modern => "Modern (120×30)",
            LayoutMode::Classic => "Classic (80×24)",
        }
    }

    /// Convert to string for config serialization
    pub fn to_string(&self) -> String {
        match self {
            LayoutMode::FourK => "FourK".to_string(),
            LayoutMode::FullHD => "FullHD".to_string(),
            LayoutMode::Modern => "Modern".to_string(),
            LayoutMode::Classic => "Classic".to_string(),
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fourk" | "4k" => Some(LayoutMode::FourK),
            "fullhd" | "full_hd" | "full-hd" => Some(LayoutMode::FullHD),
            "modern" => Some(LayoutMode::Modern),
            "classic" => Some(LayoutMode::Classic),
            _ => None,
        }
    }
}

/// Configuration for responsive layout based on terminal size
#[derive(Debug, Clone, Copy)]
pub struct LayoutConfig {
    pub width: usize,
    pub height: usize,
    pub navbar_height: usize,
    pub status_bar_height: usize,
    pub content_height: usize,
    pub layout_mode: LayoutMode,
}

impl LayoutConfig {
    /// Create a new layout configuration from terminal dimensions
    pub fn from_terminal_size(width: usize, height: usize) -> Self {
        // Determine layout mode based on realistic terminal dimensions
        let mode = if width >= 480 && height >= 120 {
            LayoutMode::FourK      // 4K maximized terminal
        } else if width >= 240 && height >= 60 {
            LayoutMode::FullHD     // Full HD maximized terminal
        } else if width >= 120 && height >= 30 {
            LayoutMode::Modern     // Modern default terminal
        } else {
            LayoutMode::Classic    // Classic 80×24 terminal
        };

        // Allocate vertical space based on mode
        let navbar_height = match mode {
            LayoutMode::FourK => 3,    // Rich navbar
            LayoutMode::FullHD => 3,   // Full navbar
            LayoutMode::Modern => 2,   // Compact navbar
            LayoutMode::Classic => 0,  // No navbar (menu-based)
        };

        let status_bar_height = match mode {
            LayoutMode::FourK => 7,    // Rich multi-row status with debug console
            LayoutMode::FullHD => 3,   // Compact status
            LayoutMode::Modern => 2,   // Minimal status
            LayoutMode::Classic => 1,  // Tiny status
        };

        let content_height = height.saturating_sub(navbar_height + status_bar_height + 2);

        Self {
            width,
            height,
            navbar_height,
            status_bar_height,
            content_height,
            layout_mode: mode,
        }
    }

    /// Calculate visible rows for list components based on layout mode
    ///
    /// Accounts for list headers, borders, and padding
    pub fn list_visible_rows(&self) -> usize {
        match self.layout_mode {
            LayoutMode::FourK => {
                // At 480×120: content_h ~105, minus ~15 for headers = ~90 visible rows
                self.content_height.saturating_sub(15)
            }
            LayoutMode::FullHD => {
                // At 240×60: content_h ~52, minus ~8 for headers = ~44 visible rows
                self.content_height.saturating_sub(8)
            }
            LayoutMode::Modern => {
                // At 120×30: content_h ~24, minus ~4 for headers = ~20 visible rows
                self.content_height.saturating_sub(4)
            }
            LayoutMode::Classic => {
                // At 80×24: content_h ~22, minus ~5 for headers = ~17 visible rows
                self.content_height.saturating_sub(5)
            }
        }
    }

    /// Check if sidebar layouts are viable in the current mode
    pub fn supports_sidebar(&self) -> bool {
        matches!(self.layout_mode, LayoutMode::FourK | LayoutMode::FullHD) && self.width >= 300
    }

    /// Get sidebar width if supported, None otherwise
    pub fn sidebar_width(&self) -> Option<usize> {
        if self.supports_sidebar() {
            Some(self.width * 30 / 100)  // 30% for sidebar
        } else {
            None
        }
    }

    /// Get the main content width (accounting for sidebar if present)
    pub fn content_width(&self) -> usize {
        if let Some(sidebar_width) = self.sidebar_width() {
            self.width.saturating_sub(sidebar_width + 2) // 2 for padding
        } else {
            self.width
        }
    }

    /// Check if the layout mode supports multi-column layouts
    pub fn supports_multi_column(&self) -> bool {
        matches!(self.layout_mode, LayoutMode::FourK | LayoutMode::FullHD) && self.width >= 300
    }

    /// Get number of columns for card grid (Trail Viewer)
    pub fn grid_columns(&self) -> usize {
        match self.layout_mode {
            LayoutMode::FourK => 3,    // 3-column grid at 480+ width
            LayoutMode::FullHD => 2,   // 2-column grid at 240+ width
            _ => 1,                    // Single column otherwise
        }
    }

    /// Get optimal number of columns for multi-column layouts
    pub fn optimal_columns(&self) -> usize {
        if !self.supports_multi_column() {
            1
        } else if self.width >= 480 {
            3  // 4K mode
        } else if self.width >= 240 {
            2  // Full HD mode
        } else {
            1  // Modern/Classic mode
        }
    }

    /// Check if navbar should be shown
    pub fn show_navbar(&self) -> bool {
        self.navbar_height > 0
    }

    /// Check if split view (side-by-side panels) is supported
    pub fn supports_split_view(&self) -> bool {
        matches!(self.layout_mode, LayoutMode::FourK | LayoutMode::FullHD) && self.width >= 240
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        // Default to Modern mode (120×30)
        Self::from_terminal_size(120, 30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fourk_mode_480x120() {
        let layout = LayoutConfig::from_terminal_size(480, 120);
        assert_eq!(layout.layout_mode, LayoutMode::FourK);
        assert_eq!(layout.navbar_height, 3);
        assert_eq!(layout.status_bar_height, 7);
        assert!(layout.content_height > 100);
        assert!(layout.supports_sidebar());
        assert_eq!(layout.grid_columns(), 3);
        assert!(layout.show_navbar());
        assert!(layout.supports_split_view());
    }

    #[test]
    fn test_fullhd_mode_240x60() {
        let layout = LayoutConfig::from_terminal_size(240, 60);
        assert_eq!(layout.layout_mode, LayoutMode::FullHD);
        assert_eq!(layout.navbar_height, 3);
        assert_eq!(layout.status_bar_height, 3);
        assert!(layout.content_height >= 50);
        assert!(!layout.supports_sidebar());  // 240 < 300
        assert_eq!(layout.grid_columns(), 2);
        assert!(layout.supports_split_view());
    }

    #[test]
    fn test_modern_mode_120x30() {
        let layout = LayoutConfig::from_terminal_size(120, 30);
        assert_eq!(layout.layout_mode, LayoutMode::Modern);
        assert_eq!(layout.navbar_height, 2);
        assert_eq!(layout.status_bar_height, 2);
        assert!(!layout.supports_sidebar());
        assert_eq!(layout.sidebar_width(), None);
        assert_eq!(layout.grid_columns(), 1);
        assert!(!layout.supports_split_view());
    }

    #[test]
    fn test_classic_mode_80x24() {
        let layout = LayoutConfig::from_terminal_size(80, 24);
        assert_eq!(layout.layout_mode, LayoutMode::Classic);
        assert_eq!(layout.navbar_height, 0); // navbar hidden in classic mode
        assert!(!layout.show_navbar());
        assert_eq!(layout.status_bar_height, 1);
        assert!(!layout.supports_sidebar());
        assert!(!layout.supports_split_view());
    }

    #[test]
    fn test_list_visible_rows() {
        // 4K mode should show large lists
        let fourk = LayoutConfig::from_terminal_size(480, 120);
        assert!(fourk.list_visible_rows() > 80);

        // Full HD mode should show substantial lists
        let fullhd = LayoutConfig::from_terminal_size(240, 60);
        assert!(fullhd.list_visible_rows() >= 40);

        let modern = LayoutConfig::from_terminal_size(120, 30);
        let classic = LayoutConfig::from_terminal_size(80, 24);

        // 4K should have most visible rows
        assert!(fourk.list_visible_rows() > fullhd.list_visible_rows());
        // Full HD should have more than modern
        assert!(fullhd.list_visible_rows() > modern.list_visible_rows());
        // Modern should have more than classic
        assert!(modern.list_visible_rows() > classic.list_visible_rows());
    }

    #[test]
    fn test_content_dimensions() {
        let layout = LayoutConfig::from_terminal_size(480, 120);
        assert_eq!(layout.width, 480);
        assert_eq!(layout.height, 120);

        // Content width should be less than total width when sidebar is present
        if layout.supports_sidebar() {
            assert!(layout.content_width() < layout.width);
        }
    }

    #[test]
    fn test_multi_column_support() {
        let classic = LayoutConfig::from_terminal_size(80, 24);
        assert!(!classic.supports_multi_column());
        assert_eq!(classic.optimal_columns(), 1);

        let fullhd = LayoutConfig::from_terminal_size(350, 60);
        assert!(fullhd.supports_multi_column());
        assert_eq!(fullhd.optimal_columns(), 2);

        let fourk = LayoutConfig::from_terminal_size(480, 120);
        assert!(fourk.supports_multi_column());
        assert_eq!(fourk.optimal_columns(), 3);
    }

    #[test]
    fn test_default_layout() {
        let layout = LayoutConfig::default();
        assert_eq!(layout.layout_mode, LayoutMode::Modern);
        assert_eq!(layout.width, 120);
        assert_eq!(layout.height, 30);
    }

    #[test]
    fn test_boundary_conditions() {
        // Test exact boundary for 4K mode
        let boundary_fourk = LayoutConfig::from_terminal_size(480, 120);
        assert_eq!(boundary_fourk.layout_mode, LayoutMode::FourK);

        // Just below boundary should be FullHD
        let boundary_fullhd = LayoutConfig::from_terminal_size(479, 119);
        assert_eq!(boundary_fullhd.layout_mode, LayoutMode::FullHD);

        // Test exact boundary for Full HD mode
        let boundary_fullhd2 = LayoutConfig::from_terminal_size(240, 60);
        assert_eq!(boundary_fullhd2.layout_mode, LayoutMode::FullHD);

        // Just below should be Modern
        let boundary_modern = LayoutConfig::from_terminal_size(239, 59);
        assert_eq!(boundary_modern.layout_mode, LayoutMode::Modern);

        // Test exact boundary for Modern mode
        let boundary_modern2 = LayoutConfig::from_terminal_size(120, 30);
        assert_eq!(boundary_modern2.layout_mode, LayoutMode::Modern);

        // Just below should be Classic
        let boundary_classic = LayoutConfig::from_terminal_size(119, 29);
        assert_eq!(boundary_classic.layout_mode, LayoutMode::Classic);
    }
}
