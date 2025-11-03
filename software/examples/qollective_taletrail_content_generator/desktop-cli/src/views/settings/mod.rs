use iocraft::prelude::*;
use crate::components::form::TextInput;
use crate::state::{SettingsContext, SettingsSection, SettingsTab, AppContext};
use crate::layout::LayoutMode;

/// Props for the Settings view
#[derive(Props)]
pub struct SettingsViewProps {
    pub settings_ctx: SettingsContext,
    pub app_context: Option<AppContext>,
}

impl Default for SettingsViewProps {
    fn default() -> Self {
        Self {
            settings_ctx: SettingsContext::new(),
            app_context: None,
        }
    }
}

/// Main settings view component
#[component]
pub fn SettingsView(_hooks: Hooks, props: &SettingsViewProps) -> impl Into<AnyElement<'static>> {
    let editing = props.settings_ctx.get_editing_config();
    let errors = props.settings_ctx.get_validation_errors();
    let is_dirty = props.settings_ctx.is_dirty();
    let active_tab = props.settings_ctx.get_active_tab();
    let current_section = props.settings_ctx.get_current_section();

    // Build the main view
    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin_bottom: 2,
            ) {
                Text(
                    content: "Settings Configuration",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
                Text(
                    content: if is_dirty { "* Unsaved Changes" } else { "Saved" },
                    color: if is_dirty { Color::Yellow } else { Color::Green }
                )
            }
        }
        .into_any(),
    );

    // Tab navigation (Overview vs Config)
    elements.push(render_tab_bar(active_tab).into());

    // Tab content
    match active_tab {
        SettingsTab::Overview => {
            // Section tabs (only in Overview tab)
            elements.push(
                element! {
                    View(margin_bottom: 2) {
                        Text(
                            content: format!(
                                "[1] NATS Connection  [2] Directories  [3] UI Preferences  (Current: {})",
                                match current_section {
                                    SettingsSection::NatsConnection => "NATS",
                                    SettingsSection::Directories => "Directories",
                                    SettingsSection::UiPreferences => "UI",
                                }
                            ),
                            color: Color::DarkGrey
                        )
                    }
                }
                .into_any(),
            );

            // Render current section
            match current_section {
                SettingsSection::NatsConnection => {
                    elements.push(render_nats_section(&editing, &errors).into());
                }
                SettingsSection::Directories => {
                    elements.push(render_directories_section(&editing, &errors).into());
                }
                SettingsSection::UiPreferences => {
                    elements.push(render_ui_section(&editing, props.app_context.as_ref()).into());
                }
            }
        }
        SettingsTab::Config => {
            // Render TOML config viewer
            elements.push(render_config_tab(&props.settings_ctx).into());
        }
    }

    // Footer with instructions
    elements.push(
        element! {
            View(
                margin_top: 2,
                padding: 1,
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey
            ) {
                Text(
                    content: "Press Ctrl+S to save | Tab to switch tabs/sections | ESC to exit",
                    color: Color::DarkGrey
                )
            }
        }
        .into_any(),
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
            border_color: Color::Magenta,
            padding: 2
        ) {
            #(elements.into_iter())
        }
    }
}

/// Render tab navigation bar
fn render_tab_bar(active_tab: SettingsTab) -> impl Into<AnyElement<'static>> {
    let tabs = [SettingsTab::Overview, SettingsTab::Config];

    let mut tab_elements: Vec<AnyElement> = Vec::new();

    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = *tab == active_tab;

        let label = format!(
            "[{}] {}",
            tab.tab_number(),
            tab.display_name()
        );

        let text_color = if is_active {
            Color::Black
        } else {
            Color::White
        };

        let bg_color = if is_active {
            Some(Color::Magenta)
        } else {
            None
        };

        let border_color = if is_active {
            Color::Magenta
        } else {
            Color::DarkGrey
        };

        tab_elements.push(
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: border_color,
                    background_color: bg_color,
                    padding: 1,
                    margin_right: if idx < tabs.len() - 1 { 1 } else { 0 },
                ) {
                    Text(content: label, color: text_color, weight: Weight::Bold)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Row,
            margin_bottom: 1,
        ) {
            #(tab_elements.into_iter())
        }
    }
}

/// Render config TOML viewer tab
fn render_config_tab(settings_ctx: &SettingsContext) -> impl Into<AnyElement<'static>> {
    let config_toml = settings_ctx.get_config_toml();

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin_bottom: 1,
            ) {
                Text(
                    content: "Configuration (TOML)",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
                Text(
                    content: "[Press 'C' to copy to clipboard]",
                    color: Color::DarkGrey
                )
            }
        }
        .into_any(),
    );

    // Config content in bordered box
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey,
                padding: 1,
            ) {
                Text(content: config_toml)
            }
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
}

/// Render NATS connection settings section
fn render_nats_section(
    editing: &crate::state::EditingConfig,
    errors: &crate::state::ValidationErrors,
) -> impl Into<AnyElement<'static>> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Section title
    elements.push(
        element! {
            View(margin_bottom: 2) {
                Text(
                    content: "=== NATS Connection Settings ===",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
            }
        }
        .into_any(),
    );

    // NATS URL field
    elements.push(
        element! {
            TextInput(
                label: "NATS URL".to_string(),
                value: editing.nats_url.clone(),
                error: errors.nats_url.clone(),
                placeholder: Some("nats://localhost:4222".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    // Timeout field
    elements.push(
        element! {
            TextInput(
                label: "Timeout (seconds)".to_string(),
                value: editing.nats_timeout.clone(),
                error: errors.nats_timeout.clone(),
                placeholder: Some("30".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    // TLS cert path field (optional)
    elements.push(
        element! {
            TextInput(
                label: "TLS Cert Path (optional)".to_string(),
                value: editing.tls_cert_path.clone(),
                error: errors.tls_cert_path.clone(),
                placeholder: Some("/path/to/cert.pem".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    // NKey path field (optional)
    elements.push(
        element! {
            TextInput(
                label: "NKey Path (optional)".to_string(),
                value: editing.nkey_path.clone(),
                error: errors.nkey_path.clone(),
                placeholder: Some("/path/to/nkey.seed".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
}

/// Render directories settings section
fn render_directories_section(
    editing: &crate::state::EditingConfig,
    errors: &crate::state::ValidationErrors,
) -> impl Into<AnyElement<'static>> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Section title
    elements.push(
        element! {
            View(margin_bottom: 2) {
                Text(
                    content: "=== Directory Settings ===",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
            }
        }
        .into_any(),
    );

    // Trails directory field
    elements.push(
        element! {
            TextInput(
                label: "Trails Directory".to_string(),
                value: editing.trails_dir.clone(),
                error: errors.trails_dir.clone(),
                placeholder: Some("./trails".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    // Templates directory field
    elements.push(
        element! {
            TextInput(
                label: "Templates Directory".to_string(),
                value: editing.templates_dir.clone(),
                error: errors.templates_dir.clone(),
                placeholder: Some("./templates".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    // Execution logs directory field
    elements.push(
        element! {
            TextInput(
                label: "Execution Logs Directory".to_string(),
                value: editing.execution_logs_dir.clone(),
                error: errors.execution_logs_dir.clone(),
                placeholder: Some("./execution_logs".to_string()),
                is_focused: false
            )
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
}

/// Render UI preferences section
fn render_ui_section(
    editing: &crate::state::EditingConfig,
    app_context: Option<&AppContext>,
) -> impl Into<AnyElement<'static>> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Section title
    elements.push(
        element! {
            View(margin_bottom: 2) {
                Text(
                    content: "=== UI Preferences ===",
                    weight: Weight::Bold,
                    color: Color::Cyan
                )
            }
        }
        .into_any(),
    );

    // Display Mode section
    if let Some(ctx) = app_context {
        let environment = ctx.environment();
        let manual_mode = ctx.manual_display_mode();
        let current_layout = ctx.layout_config();
        let terminal_width = ctx.terminal_width();
        let terminal_height = ctx.terminal_height();

        elements.push(
            element! {
                View(
                    margin_bottom: 2,
                    border_style: BorderStyle::Single,
                    border_color: Color::DarkGrey,
                    padding: 1
                ) {
                    Text(
                        content: "Display Mode",
                        weight: Weight::Bold,
                        color: Color::Cyan
                    )
                }
            }
            .into_any(),
        );

        let mode_text = if let Some(mode) = manual_mode {
            format!("Manual: {}", mode.display_name())
        } else {
            format!("Auto (detected: {})", current_layout.layout_mode.display_name())
        };

        elements.push(
            element! {
                View(margin_bottom: 1) {
                    Text(
                        content: format!("Current Mode: {}", mode_text),
                        color: Color::White
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1) {
                    Text(
                        content: format!("Detected Terminal: {}×{}", terminal_width, terminal_height),
                        color: Color::DarkGrey
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1) {
                    Text(
                        content: format!("Environment: {}", environment.name()),
                        color: Color::DarkGrey
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 2) {
                    Text(
                        content: "Options: [Ctrl+L to cycle modes]",
                        color: Color::Yellow
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1, padding_left: 2) {
                    Text(
                        content: "- Auto: Detect from terminal size",
                        color: if manual_mode.is_none() { Color::Green } else { Color::White }
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1, padding_left: 2) {
                    Text(
                        content: "- Classic (80×24)",
                        color: if manual_mode == Some(LayoutMode::Classic) { Color::Green } else { Color::White }
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1, padding_left: 2) {
                    Text(
                        content: "- Modern (120×30)",
                        color: if manual_mode == Some(LayoutMode::Modern) { Color::Green } else { Color::White }
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 1, padding_left: 2) {
                    Text(
                        content: "- Full HD (240×60)",
                        color: if manual_mode == Some(LayoutMode::FullHD) { Color::Green } else { Color::White }
                    )
                }
            }
            .into_any(),
        );

        elements.push(
            element! {
                View(margin_bottom: 2, padding_left: 2) {
                    Text(
                        content: "- 4K (480×120)",
                        color: if manual_mode == Some(LayoutMode::FourK) { Color::Green } else { Color::White }
                    )
                }
            }
            .into_any(),
        );
    }

    // Color theme display
    elements.push(
        element! {
            View(margin_bottom: 2) {
                Text(
                    content: format!("Color Theme: {}", editing.color_theme),
                    color: Color::White
                )
                Text(
                    content: "  (Press 'T' to toggle between dark/light)",
                    color: Color::DarkGrey
                )
            }
        }
        .into_any(),
    );

    // Auto-scroll display
    elements.push(
        element! {
            View(margin_bottom: 2) {
                Text(
                    content: format!(
                        "Auto-scroll: {}",
                        if editing.auto_scroll { "Enabled" } else { "Disabled" }
                    ),
                    color: Color::White
                )
                Text(
                    content: "  (Press 'A' to toggle)",
                    color: Color::DarkGrey
                )
            }
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_settings_view_props_creation() {
        let ctx = SettingsContext::new();
        let _props = SettingsViewProps { settings_ctx: ctx };
        // If this compiles, the test passes
    }

    #[test]
    fn test_settings_view_with_custom_config() {
        let mut config = Config::default();
        config.nats.url = "nats://custom:4222".to_string();

        let ctx = SettingsContext::from_config(config);
        let props = SettingsViewProps { settings_ctx: ctx };

        let editing = props.settings_ctx.get_editing_config();
        assert_eq!(editing.nats_url, "nats://custom:4222");
    }

    #[test]
    fn test_settings_section_switching() {
        let ctx = SettingsContext::new();

        // Start on NATS section
        assert_eq!(ctx.get_current_section(), SettingsSection::NatsConnection);

        // Switch to directories
        ctx.set_current_section(SettingsSection::Directories);
        assert_eq!(ctx.get_current_section(), SettingsSection::Directories);

        // Switch to UI
        ctx.set_current_section(SettingsSection::UiPreferences);
        assert_eq!(ctx.get_current_section(), SettingsSection::UiPreferences);
    }
}
