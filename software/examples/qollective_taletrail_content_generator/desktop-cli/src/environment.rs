/// Environment detection and capabilities for TaleTrail Desktop CLI
///
/// Detects the runtime environment (iTerm2, RustRover IDE, or Unknown terminal)
/// and provides capability information for adaptive UI rendering.

use std::env;

/// The detected runtime environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Running in iTerm2 terminal emulator
    ITerm2,
    /// Running in RustRover IDE console
    RustRover,
    /// Running in an unknown/generic terminal
    Unknown,
}

impl Environment {
    /// Detect the current runtime environment by examining environment variables
    pub fn detect() -> Self {
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let inside_idea = env::var("IDEA_INITIAL_DIRECTORY").is_ok();
        let inside_rustover = env::var("INSIDE_RUSTOVER").is_ok();

        if term_program == "iTerm.app" {
            Environment::ITerm2
        } else if inside_idea || inside_rustover {
            Environment::RustRover
        } else {
            Environment::Unknown
        }
    }

    /// Get the capabilities available in this environment
    pub fn capabilities(&self) -> Capabilities {
        match self {
            Environment::ITerm2 => Capabilities {
                mouse_support: true,
                true_color: true,
                alternate_screen: true,
                unicode_support: true,
                emoji_support: true,
                resize_events: true,
            },
            Environment::RustRover => Capabilities {
                mouse_support: false,
                true_color: false,
                alternate_screen: false,
                unicode_support: true,
                emoji_support: false,
                resize_events: false,
            },
            Environment::Unknown => Capabilities::minimal(),
        }
    }

    /// Returns true if this is an IDE console environment
    pub fn is_ide(&self) -> bool {
        matches!(self, Environment::RustRover)
    }

    /// Returns a human-readable name for this environment
    pub fn name(&self) -> &'static str {
        match self {
            Environment::ITerm2 => "iTerm2",
            Environment::RustRover => "RustRover (IDE)",
            Environment::Unknown => "Unknown Terminal",
        }
    }
}

/// Terminal capabilities available in the current environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Mouse events are supported and functional
    pub mouse_support: bool,
    /// 24-bit true color is supported
    pub true_color: bool,
    /// Alternate screen buffer is supported
    pub alternate_screen: bool,
    /// Full Unicode support including box drawing characters
    pub unicode_support: bool,
    /// Emoji rendering is supported
    pub emoji_support: bool,
    /// Terminal resize events are properly sent
    pub resize_events: bool,
}

impl Capabilities {
    /// Returns minimal capabilities for unknown/limited terminals
    pub fn minimal() -> Self {
        Self {
            mouse_support: false,
            true_color: false,
            alternate_screen: true,
            unicode_support: true,
            emoji_support: false,
            resize_events: true,
        }
    }

    /// Detect color support level from environment variables
    pub fn detect_color_support() -> ColorSupport {
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        let term = env::var("TERM").unwrap_or_default();

        if colorterm == "truecolor" || colorterm == "24bit" {
            ColorSupport::TrueColor
        } else if term.contains("256") {
            ColorSupport::Ansi256
        } else {
            ColorSupport::Ansi16
        }
    }
}

/// Level of color support available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    /// 24-bit RGB true color
    TrueColor,
    /// 256-color palette
    Ansi256,
    /// 16-color ANSI
    Ansi16,
}

impl ColorSupport {
    /// Returns a human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ColorSupport::TrueColor => "True Color (24-bit)",
            ColorSupport::Ansi256 => "256 Colors",
            ColorSupport::Ansi16 => "16 Colors (ANSI)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        let env = Environment::detect();
        assert!(matches!(
            env,
            Environment::ITerm2 | Environment::RustRover | Environment::Unknown
        ));
    }

    #[test]
    fn test_capabilities() {
        let iterm_caps = Environment::ITerm2.capabilities();
        assert!(iterm_caps.mouse_support);
        assert!(iterm_caps.true_color);

        let ide_caps = Environment::RustRover.capabilities();
        assert!(!ide_caps.mouse_support);
        assert!(!ide_caps.true_color);
    }

    #[test]
    fn test_minimal_capabilities() {
        let caps = Capabilities::minimal();
        assert!(!caps.mouse_support);
        assert!(!caps.true_color);
    }
}
