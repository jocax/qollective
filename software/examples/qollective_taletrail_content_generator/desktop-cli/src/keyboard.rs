use iocraft::prelude::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::HashMap;
use std::time::Instant;

use crate::state::View;

/// Represents a parsed keyboard binding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyBinding {
    /// Single key with modifiers: "alt-1", "ctrl-s", "f2"
    Single(KeyCode, KeyModifiers),

    /// Vim-style sequence: "g 1" = [Char('g'), Char('1')]
    Sequence(Vec<KeyCode>),

    /// Character with modifiers (normalized to lowercase): "ctrl-s"
    Character(char, KeyModifiers),
}

impl KeyBinding {
    /// Parse key notation string into KeyBinding
    /// Examples: "alt-1", "ctrl-shift-t", "g 1", "enter", "f12"
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        // 1. Handle sequences (contains space): "g 1" -> Sequence
        if s.contains(' ') {
            let parts: Vec<&str> = s.split(' ').collect();
            let keys: Result<Vec<KeyCode>, ParseError> =
                parts.iter().map(|p| Self::parse_key_code(p)).collect();
            return Ok(KeyBinding::Sequence(keys?));
        }

        // 2. Handle modifiers (contains dash): "ctrl-s" -> Single or Character
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            let mut modifiers = KeyModifiers::empty();
            let mut key_part = "";

            for part in parts {
                match part.to_lowercase().as_str() {
                    "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                    "alt" | "option" | "meta" => modifiers |= KeyModifiers::ALT,
                    "shift" => modifiers |= KeyModifiers::SHIFT,
                    // "cmd" ignored on non-macOS
                    _ => key_part = part,
                }
            }

            // If single character, use Character variant
            if key_part.len() == 1 {
                let ch = key_part.chars().next().unwrap();
                return Ok(KeyBinding::Character(
                    ch.to_lowercase().next().unwrap(),
                    modifiers,
                ));
            } else {
                let key_code = Self::parse_key_code(key_part)?;
                return Ok(KeyBinding::Single(key_code, modifiers));
            }
        }

        // 3. Handle plain keys: "a", "enter", "f1"
        if s.len() == 1 {
            let ch = s.chars().next().unwrap();
            Ok(KeyBinding::Character(
                ch.to_lowercase().next().unwrap(),
                KeyModifiers::empty(),
            ))
        } else {
            Ok(KeyBinding::Single(
                Self::parse_key_code(s)?,
                KeyModifiers::empty(),
            ))
        }
    }

    fn parse_key_code(s: &str) -> Result<KeyCode, ParseError> {
        match s.to_lowercase().as_str() {
            "enter" | "return" => Ok(KeyCode::Enter),
            "tab" => Ok(KeyCode::Tab),
            "backspace" => Ok(KeyCode::Backspace),
            "delete" | "del" => Ok(KeyCode::Delete),
            "esc" | "escape" => Ok(KeyCode::Esc),
            "space" => Ok(KeyCode::Char(' ')),
            "left" => Ok(KeyCode::Left),
            "right" => Ok(KeyCode::Right),
            "up" => Ok(KeyCode::Up),
            "down" => Ok(KeyCode::Down),
            s if s.starts_with('f') => s[1..]
                .parse::<u8>()
                .map(KeyCode::F)
                .map_err(|_| ParseError::InvalidFunctionKey(s.to_string())),
            s if s.len() == 1 => Ok(KeyCode::Char(s.chars().next().unwrap())),
            _ => Err(ParseError::UnknownKey(s.to_string())),
        }
    }

    /// Format this binding as a human-readable string
    pub fn format(&self) -> String {
        match self {
            KeyBinding::Character(ch, mods) => {
                let mut result = String::new();
                if mods.contains(KeyModifiers::CONTROL) {
                    result.push_str("Ctrl+");
                }
                if mods.contains(KeyModifiers::ALT) {
                    #[cfg(target_os = "macos")]
                    result.push_str("⌥");
                    #[cfg(not(target_os = "macos"))]
                    result.push_str("Alt+");
                }
                if mods.contains(KeyModifiers::SHIFT) {
                    result.push_str("Shift+");
                }
                result.push(ch.to_uppercase().next().unwrap());
                result
            }
            KeyBinding::Single(KeyCode::F(n), _) => format!("F{}", n),
            KeyBinding::Single(code, mods) => {
                let mut result = String::new();
                if mods.contains(KeyModifiers::CONTROL) {
                    result.push_str("Ctrl+");
                }
                if mods.contains(KeyModifiers::ALT) {
                    #[cfg(target_os = "macos")]
                    result.push_str("⌥");
                    #[cfg(not(target_os = "macos"))]
                    result.push_str("Alt+");
                }
                if mods.contains(KeyModifiers::SHIFT) {
                    result.push_str("Shift+");
                }
                result.push_str(&format!("{:?}", code));
                result
            }
            KeyBinding::Sequence(seq) => seq
                .iter()
                .map(|k| format!("{:?}", k).replace("Char('", "").replace("')", ""))
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnknownKey(String),
    InvalidFunctionKey(String),
    InvalidFormat(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownKey(k) => write!(f, "Unknown key: {}", k),
            ParseError::InvalidFunctionKey(k) => write!(f, "Invalid function key: {}", k),
            ParseError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// All possible actions in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Global actions (work in any view)
    Help,
    Quit,
    DebugMode,
    DebugConsole,
    ThemeToggle,
    DisplayModeCycle,

    // Navigation actions (jump to specific view)
    GoToView(View),

    // Menu actions (context-specific, only in Menu view)
    MenuNavigateUp,
    MenuNavigateDown,
    MenuSelect,
    MenuBack,
}

impl Action {
    /// Parse action string from config
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "help" => Some(Action::Help),
            "quit" => Some(Action::Quit),
            "debug_mode" => Some(Action::DebugMode),
            "debug_console" => Some(Action::DebugConsole),
            "theme_toggle" => Some(Action::ThemeToggle),
            "display_mode_cycle" => Some(Action::DisplayModeCycle),

            "mcp_tester" => Some(Action::GoToView(View::McpTester)),
            "trail_viewer" => Some(Action::GoToView(View::TrailViewer)),
            "nats_monitor" => Some(Action::GoToView(View::NatsMonitor)),
            "story_generator" => Some(Action::GoToView(View::StoryGenerator)),
            "search" => Some(Action::GoToView(View::Search)),
            "settings" => Some(Action::GoToView(View::Settings)),
            "logs" => Some(Action::GoToView(View::Logs)),

            "menu_navigate_up" => Some(Action::MenuNavigateUp),
            "menu_navigate_down" => Some(Action::MenuNavigateDown),
            "menu_select" => Some(Action::MenuSelect),
            "menu_back" => Some(Action::MenuBack),

            _ => None,
        }
    }

    /// Get display name for this action
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::Help => "Help",
            Action::Quit => "Quit",
            Action::DebugMode => "Debug Mode",
            Action::DebugConsole => "Debug Console",
            Action::ThemeToggle => "Theme Toggle",
            Action::DisplayModeCycle => "Display Mode Cycle",
            Action::GoToView(view) => view.display_name(),
            Action::MenuNavigateUp => "Navigate Up",
            Action::MenuNavigateDown => "Navigate Down",
            Action::MenuSelect => "Select",
            Action::MenuBack => "Back",
        }
    }
}

pub struct KeyMapper {
    bindings: HashMap<Action, Vec<KeyBinding>>,
    sequence_buffer: Vec<KeyCode>,
    sequence_timeout: Option<Instant>,
    timeout_ms: u64,
}

impl KeyMapper {
    /// Create KeyMapper from KeyBindings config
    pub fn from_config(config: crate::config::KeyBindings) -> Result<Self, ValidationError> {
        let mut bindings: HashMap<Action, Vec<KeyBinding>> = HashMap::new();

        // Parse global bindings
        for (action_str, key_strs) in config.global {
            let action = Action::parse(&action_str)
                .ok_or_else(|| ValidationError::UnknownAction(action_str.clone()))?;

            let keys: Result<Vec<KeyBinding>, ParseError> =
                key_strs.iter().map(|s| KeyBinding::parse(s)).collect();

            bindings.insert(action, keys.map_err(ValidationError::ParseError)?);
        }

        // Parse navigation bindings
        for (action_str, key_strs) in config.navigation {
            let action = Action::parse(&action_str)
                .ok_or_else(|| ValidationError::UnknownAction(action_str.clone()))?;

            let keys: Result<Vec<KeyBinding>, ParseError> =
                key_strs.iter().map(|s| KeyBinding::parse(s)).collect();

            bindings.insert(action, keys.map_err(ValidationError::ParseError)?);
        }

        // Parse menu bindings
        for (action_str, key_strs) in config.menu {
            let action = Action::parse(&action_str)
                .ok_or_else(|| ValidationError::UnknownAction(action_str.clone()))?;

            let keys: Result<Vec<KeyBinding>, ParseError> =
                key_strs.iter().map(|s| KeyBinding::parse(s)).collect();

            bindings.insert(action, keys.map_err(ValidationError::ParseError)?);
        }

        // Validate all required actions have bindings
        Self::validate_required_actions(&bindings)?;

        Ok(Self {
            bindings,
            sequence_buffer: Vec::new(),
            sequence_timeout: None,
            timeout_ms: config.sequence_timeout_ms.unwrap_or(500),
        })
    }

    fn validate_required_actions(
        bindings: &HashMap<Action, Vec<KeyBinding>>,
    ) -> Result<(), ValidationError> {
        // Ensure critical actions have bindings
        let required = vec![Action::Help, Action::Quit];

        for action in required {
            if !bindings.contains_key(&action) || bindings[&action].is_empty() {
                return Err(ValidationError::MissingRequiredAction(
                    action.display_name().to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Handle incoming keyboard event, return matched action if any
    pub fn handle_event(&mut self, event: &KeyEvent) -> Option<Action> {
        // Only process key press, not release or repeat
        if event.kind != KeyEventKind::Press {
            return None;
        }

        let now = Instant::now();

        // 1. Check sequence timeout - clear buffer if expired
        if let Some(timeout) = self.sequence_timeout {
            if now.duration_since(timeout).as_millis() > self.timeout_ms as u128 {
                self.sequence_buffer.clear();
                self.sequence_timeout = None;
            }
        }

        // 2. If sequence buffer is active, try to complete sequence
        if !self.sequence_buffer.is_empty() {
            self.sequence_buffer.push(event.code);

            // Check all bindings for matching sequence
            if let Some(action) = self.match_sequence() {
                self.sequence_buffer.clear();
                self.sequence_timeout = None;
                return Some(action);
            }

            // Update timeout and continue waiting
            self.sequence_timeout = Some(now);

            // Clear buffer if Escape pressed or too long
            if event.code == KeyCode::Esc || self.sequence_buffer.len() > 5 {
                self.sequence_buffer.clear();
                self.sequence_timeout = None;
            }

            return None;
        }

        // 3. Try exact single-key match
        if let Some(action) = self.match_single(event) {
            return Some(action);
        }

        // 4. Check if this could start a sequence
        if self.could_start_sequence(event.code) {
            self.sequence_buffer.push(event.code);
            self.sequence_timeout = Some(now);
        }

        None
    }

    fn match_single(&self, key_event: &KeyEvent) -> Option<Action> {
        for (action, bindings) in &self.bindings {
            for binding in bindings {
                match binding {
                    KeyBinding::Single(code, mods) => {
                        if key_event.code == *code && key_event.modifiers == *mods {
                            return Some(*action);
                        }
                    }
                    KeyBinding::Character(ch, mods) => {
                        if let KeyCode::Char(c) = key_event.code {
                            if c.to_lowercase().next() == Some(*ch)
                                && key_event.modifiers == *mods
                            {
                                return Some(*action);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn match_sequence(&self) -> Option<Action> {
        for (action, bindings) in &self.bindings {
            for binding in bindings {
                if let KeyBinding::Sequence(seq) = binding {
                    if seq == &self.sequence_buffer {
                        return Some(*action);
                    }
                }
            }
        }
        None
    }

    fn could_start_sequence(&self, code: KeyCode) -> bool {
        for bindings in self.bindings.values() {
            for binding in bindings {
                if let KeyBinding::Sequence(seq) = binding {
                    if !seq.is_empty() && seq[0] == code {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get all bindings for a specific action (for UI display)
    pub fn get_bindings_for(&self, action: Action) -> Option<&Vec<KeyBinding>> {
        self.bindings.get(&action)
    }

    /// Get sequence buffer state (for debugging)
    pub fn sequence_buffer_state(&self) -> &[KeyCode] {
        &self.sequence_buffer
    }
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    UnknownAction(String),
    ParseError(ParseError),
    MissingRequiredAction(String),
    ConflictingBinding(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::UnknownAction(a) => write!(f, "Unknown action: {}", a),
            ValidationError::ParseError(e) => write!(f, "Parse error: {}", e),
            ValidationError::MissingRequiredAction(a) => {
                write!(f, "Missing required action: {}", a)
            }
            ValidationError::ConflictingBinding(b) => write!(f, "Conflicting binding: {}", b),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_key() {
        let binding = KeyBinding::parse("a").unwrap();
        assert_eq!(binding, KeyBinding::Character('a', KeyModifiers::empty()));
    }

    #[test]
    fn test_parse_modified_key() {
        let binding = KeyBinding::parse("alt-1").unwrap();
        assert!(matches!(binding, KeyBinding::Character('1', _)));
        if let KeyBinding::Character(_, mods) = binding {
            assert!(mods.contains(KeyModifiers::ALT));
        }
    }

    #[test]
    fn test_parse_sequence() {
        let binding = KeyBinding::parse("g 1").unwrap();
        if let KeyBinding::Sequence(seq) = binding {
            assert_eq!(seq.len(), 2);
            assert_eq!(seq[0], KeyCode::Char('g'));
            assert_eq!(seq[1], KeyCode::Char('1'));
        } else {
            panic!("Expected Sequence variant");
        }
    }

    #[test]
    fn test_parse_function_key() {
        let binding = KeyBinding::parse("f12").unwrap();
        assert_eq!(
            binding,
            KeyBinding::Single(KeyCode::F(12), KeyModifiers::empty())
        );
    }

    #[test]
    fn test_parse_special_keys() {
        assert_eq!(
            KeyBinding::parse("enter").unwrap(),
            KeyBinding::Single(KeyCode::Enter, KeyModifiers::empty())
        );
        assert_eq!(
            KeyBinding::parse("esc").unwrap(),
            KeyBinding::Single(KeyCode::Esc, KeyModifiers::empty())
        );
        assert_eq!(
            KeyBinding::parse("tab").unwrap(),
            KeyBinding::Single(KeyCode::Tab, KeyModifiers::empty())
        );
    }

    #[test]
    fn test_parse_invalid_key() {
        assert!(KeyBinding::parse("invalid").is_err());
        assert!(KeyBinding::parse("f999").is_err());
    }

    #[test]
    fn test_parse_ctrl_key() {
        let binding = KeyBinding::parse("ctrl-q").unwrap();
        if let KeyBinding::Character(ch, mods) = binding {
            assert_eq!(ch, 'q');
            assert!(mods.contains(KeyModifiers::CONTROL));
        } else {
            panic!("Expected Character variant");
        }
    }

    #[test]
    fn test_parse_ctrl_shift_key() {
        let binding = KeyBinding::parse("ctrl-shift-t").unwrap();
        if let KeyBinding::Character(ch, mods) = binding {
            assert_eq!(ch, 't');
            assert!(mods.contains(KeyModifiers::CONTROL));
            assert!(mods.contains(KeyModifiers::SHIFT));
        } else {
            panic!("Expected Character variant");
        }
    }

    #[test]
    fn test_action_parse() {
        assert_eq!(Action::parse("help"), Some(Action::Help));
        assert_eq!(Action::parse("quit"), Some(Action::Quit));
        assert_eq!(
            Action::parse("mcp_tester"),
            Some(Action::GoToView(View::McpTester))
        );
        assert_eq!(Action::parse("invalid"), None);
    }

    #[test]
    fn test_action_display_name() {
        assert_eq!(Action::Help.display_name(), "Help");
        assert_eq!(Action::Quit.display_name(), "Quit");
        assert_eq!(
            Action::GoToView(View::McpTester).display_name(),
            "MCP Tester"
        );
    }

    #[test]
    fn test_key_binding_format() {
        let binding = KeyBinding::Character('q', KeyModifiers::CONTROL);
        let formatted = binding.format();
        assert!(formatted.contains("Ctrl"));
        assert!(formatted.contains("Q"));
    }

    #[test]
    fn test_key_binding_format_function_key() {
        let binding = KeyBinding::Single(KeyCode::F(12), KeyModifiers::empty());
        assert_eq!(binding.format(), "F12");
    }

    #[test]
    fn test_key_binding_format_sequence() {
        let binding = KeyBinding::Sequence(vec![KeyCode::Char('g'), KeyCode::Char('1')]);
        let formatted = binding.format();
        assert!(formatted.contains("g"));
        assert!(formatted.contains("1"));
    }

    #[test]
    fn test_keymapper_match_single() {
        let mut config = crate::config::KeyBindings::default_for_platform();

        // Add a simple binding for testing
        config
            .global
            .insert("help".to_string(), vec!["f1".to_string()]);
        config
            .global
            .insert("quit".to_string(), vec!["ctrl-q".to_string()]);

        let mut mapper = KeyMapper::from_config(config).unwrap();

        // Test F1 key - create KeyEvent manually with proper structure
        let mut event = KeyEvent::new(KeyEventKind::Press, KeyCode::F(1));
        event.modifiers = KeyModifiers::empty();
        assert_eq!(mapper.handle_event(&event), Some(Action::Help));

        // Test Ctrl+Q
        let mut event = KeyEvent::new(KeyEventKind::Press, KeyCode::Char('q'));
        event.modifiers = KeyModifiers::CONTROL;
        assert_eq!(mapper.handle_event(&event), Some(Action::Quit));
    }

    #[test]
    fn test_keymapper_validation_missing_required() {
        let config = crate::config::KeyBindings {
            platform: "auto".to_string(),
            sequence_timeout_ms: Some(500),
            global: HashMap::new(), // Missing required bindings
            menu: HashMap::new(),
            navigation: HashMap::new(),
            platforms: HashMap::new(),
        };

        let result = KeyMapper::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_keymapper_get_bindings_for() {
        let mut config = crate::config::KeyBindings::default_for_platform();
        config
            .global
            .insert("help".to_string(), vec!["f1".to_string()]);
        config
            .global
            .insert("quit".to_string(), vec!["ctrl-q".to_string()]);

        let mapper = KeyMapper::from_config(config).unwrap();

        let bindings = mapper.get_bindings_for(Action::Help);
        assert!(bindings.is_some());
        assert_eq!(bindings.unwrap().len(), 1);
    }

    #[test]
    fn test_sequence_buffer_timeout() {
        let mut config = crate::config::KeyBindings::default_for_platform();
        config.global.insert("help".to_string(), vec!["f1".to_string()]);
        config.global.insert("quit".to_string(), vec!["ctrl-q".to_string()]);
        config.sequence_timeout_ms = Some(100); // 100ms timeout

        let mapper = KeyMapper::from_config(config).unwrap();

        // Sequence buffer should be empty initially
        assert_eq!(mapper.sequence_buffer_state().len(), 0);
    }

    #[test]
    fn test_parse_alt_key() {
        let binding = KeyBinding::parse("alt-1").unwrap();
        if let KeyBinding::Character(ch, mods) = binding {
            assert_eq!(ch, '1');
            assert!(mods.contains(KeyModifiers::ALT));
        } else {
            panic!("Expected Character variant");
        }
    }

    #[test]
    fn test_parse_option_key() {
        // "option" should be treated as ALT
        let binding = KeyBinding::parse("option-1").unwrap();
        if let KeyBinding::Character(ch, mods) = binding {
            assert_eq!(ch, '1');
            assert!(mods.contains(KeyModifiers::ALT));
        } else {
            panic!("Expected Character variant");
        }
    }

    #[test]
    fn test_case_insensitive_parsing() {
        let binding1 = KeyBinding::parse("ctrl-Q").unwrap();
        let binding2 = KeyBinding::parse("ctrl-q").unwrap();
        assert_eq!(binding1, binding2);
    }

    #[test]
    fn test_multiple_modifiers() {
        let binding = KeyBinding::parse("ctrl-alt-t").unwrap();
        if let KeyBinding::Character(ch, mods) = binding {
            assert_eq!(ch, 't');
            assert!(mods.contains(KeyModifiers::CONTROL));
            assert!(mods.contains(KeyModifiers::ALT));
        } else {
            panic!("Expected Character variant");
        }
    }
}
