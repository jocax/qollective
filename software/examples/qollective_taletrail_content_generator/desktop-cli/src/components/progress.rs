use iocraft::prelude::*;

/// Props for ProgressBar component
#[derive(Props)]
pub struct ProgressBarProps {
    pub percentage: f32,
    pub width: usize,
    pub label: Option<String>,
}

impl Default for ProgressBarProps {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            width: 40,
            label: None,
        }
    }
}

/// Progress bar component showing percentage complete
#[component]
pub fn ProgressBar(_hooks: Hooks, props: &ProgressBarProps) -> impl Into<AnyElement<'static>> {
    // Clamp percentage to 0-100
    let percentage = props.percentage.max(0.0).min(100.0);

    // Calculate filled width
    let filled_width = ((percentage / 100.0) * props.width as f32) as usize;
    let empty_width = props.width.saturating_sub(filled_width);

    // Create progress bar string
    let filled = "█".repeat(filled_width);
    let empty = "░".repeat(empty_width);
    let bar = format!("{}{}", filled, empty);

    let percentage_text = format!("{:.1}%", percentage);

    let mut elements: Vec<AnyElement> = Vec::new();

    // Optional label
    if let Some(label) = &props.label {
        elements.push(
            element! {
                Text(content: label, color: Color::Cyan)
            }
            .into_any(),
        );
    }

    // Progress bar
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(content: bar, color: Color::Cyan)
            }
        }
        .into_any(),
    );

    // Percentage text
    elements.push(
        element! {
            View(margin_top: 1) {
                Text(content: percentage_text, color: Color::White)
            }
        }
        .into_any(),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Spinner frame for indeterminate progress
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Get spinner frame for a given tick
pub fn get_spinner_frame(tick: usize) -> &'static str {
    SPINNER_FRAMES[tick % SPINNER_FRAMES.len()]
}

/// Props for Spinner component
#[derive(Props)]
pub struct SpinnerProps {
    pub tick: usize,
    pub message: Option<String>,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            tick: 0,
            message: None,
        }
    }
}

/// Spinner component for indeterminate progress
#[component]
pub fn Spinner(_hooks: Hooks, props: &SpinnerProps) -> impl Into<AnyElement<'static>> {
    let frame = get_spinner_frame(props.tick);

    if let Some(message) = &props.message {
        element! {
            View {
                Text(content: format!("{} {}", frame, message), color: Color::Cyan)
            }
        }
    } else {
        element! {
            View {
                Text(content: frame, color: Color::Cyan)
            }
        }
    }
}

/// Props for LoadingBar component (hybrid progress + spinner)
#[derive(Props)]
pub struct LoadingBarProps {
    pub percentage: Option<f32>,
    pub tick: usize,
    pub message: Option<String>,
    pub width: usize,
}

impl Default for LoadingBarProps {
    fn default() -> Self {
        Self {
            percentage: None,
            tick: 0,
            message: None,
            width: 40,
        }
    }
}

/// Loading bar that shows either determinate or indeterminate progress
#[component]
pub fn LoadingBar(_hooks: Hooks, props: &LoadingBarProps) -> impl Into<AnyElement<'static>> {
    let mut elements: Vec<AnyElement> = Vec::new();

    // Show message with spinner
    if let Some(message) = &props.message {
        let frame = get_spinner_frame(props.tick);
        elements.push(
            element! {
                View(margin_bottom: 1) {
                    Text(content: format!("{} {}", frame, message), color: Color::Cyan)
                }
            }
            .into_any(),
        );
    }

    // Show progress bar if percentage is available
    if let Some(percentage) = props.percentage {
        let percentage = percentage.max(0.0).min(100.0);
        let filled_width = ((percentage / 100.0) * props.width as f32) as usize;
        let empty_width = props.width.saturating_sub(filled_width);

        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(empty_width);
        let bar = format!("{}{}", filled, empty);

        let percentage_text = format!(" {:.1}%", percentage);

        elements.push(
            element! {
                View {
                    Text(content: format!("{}{}", bar, percentage_text), color: Color::Cyan)
                }
            }
            .into_any(),
        );
    } else {
        // Show indeterminate progress (animated dots or bar)
        let pos = props.tick % (props.width + 5);
        let mut bar_chars = vec!['░'; props.width];

        // Create moving window of filled characters
        for i in 0..5 {
            let idx = pos.saturating_sub(i);
            if idx < props.width {
                bar_chars[idx] = '█';
            }
        }

        let bar: String = bar_chars.iter().collect();

        elements.push(
            element! {
                View {
                    Text(content: bar, color: Color::Cyan)
                }
            }
            .into_any(),
        );
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: Color::Cyan,
            padding: 1,
        ) {
            #(elements.into_iter())
        }
    }
    .into_any()
}

/// Multi-step progress tracker
#[derive(Clone)]
pub struct MultiStepProgress {
    steps: Vec<(String, ProgressStatus)>,
    current_step: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProgressStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl MultiStepProgress {
    pub fn new(step_names: Vec<String>) -> Self {
        let steps = step_names
            .into_iter()
            .map(|name| (name, ProgressStatus::Pending))
            .collect();
        Self {
            steps,
            current_step: 0,
        }
    }

    pub fn start_step(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].1 = ProgressStatus::InProgress;
            self.current_step = index;
        }
    }

    pub fn complete_step(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].1 = ProgressStatus::Completed;
        }
    }

    pub fn fail_step(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].1 = ProgressStatus::Failed;
        }
    }

    pub fn status_symbol(status: &ProgressStatus) -> &'static str {
        match status {
            ProgressStatus::Pending => "○",
            ProgressStatus::InProgress => "◐",
            ProgressStatus::Completed => "●",
            ProgressStatus::Failed => "✗",
        }
    }

    pub fn status_color(status: &ProgressStatus) -> Color {
        match status {
            ProgressStatus::Pending => Color::Grey,
            ProgressStatus::InProgress => Color::Cyan,
            ProgressStatus::Completed => Color::Green,
            ProgressStatus::Failed => Color::Red,
        }
    }

    pub fn render_steps(&self) -> Vec<AnyElement<'static>> {
        self.steps
            .iter()
            .map(|(name, status)| {
                let symbol = Self::status_symbol(status);
                let color = Self::status_color(status);
                let content = format!("{} {}", symbol, name);

                element! {
                    View {
                        Text(content: content, color: color)
                    }
                }
                .into_any()
            })
            .collect()
    }
}
