/// Trail Detail View Component
///
/// Displays complete trail metadata, story structure, and execution trace

use crate::models::trail::{TrailListItem, GenerationResponse};
use iocraft::prelude::*;

/// Props for Trail Detail View
#[derive(Props)]
pub struct TrailDetailViewProps {
    pub trail: TrailListItem,
    pub full_data: Option<GenerationResponse>,
}

impl Default for TrailDetailViewProps {
    fn default() -> Self {
        Self {
            trail: TrailListItem {
                id: String::new(),
                file_path: String::new(),
                title: String::new(),
                description: String::new(),
                theme: String::new(),
                age_group: String::new(),
                language: String::new(),
                tags: Vec::new(),
                status: String::new(),
                generated_at: String::new(),
                node_count: 0,
                tenant_id: None,
            },
            full_data: None,
        }
    }
}

/// Trail Detail View Component
///
/// Displays:
/// - Trail metadata (title, description, theme, etc.)
/// - Generation parameters (age group, language, node count, etc.)
/// - Story structure as ASCII DAG tree
/// - Execution trace summary
#[component]
pub fn TrailDetailView(
    _hooks: Hooks,
    props: &TrailDetailViewProps,
) -> impl Into<AnyElement<'static>> {
    let trail = &props.trail;

    let mut elements: Vec<AnyElement> = Vec::new();

    // Header
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Double,
                border_color: Color::Green,
                padding: 2,
                margin_bottom: 1
            ) {
                Text(
                    content: format!("Trail Details: {}", trail.title),
                    color: Color::Green,
                    weight: Weight::Bold
                )
            }
        }
        .into_any(),
    );

    // Metadata section
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Cyan,
                padding: 2,
                margin_bottom: 1
            ) {
                Text(content: "Metadata", color: Color::Cyan, weight: Weight::Bold)
                Text(content: format!("ID: {}", trail.id), color: Color::White)
                Text(content: format!("Description: {}", trail.description), color: Color::White)
                Text(content: format!("Theme: {}", trail.theme), color: Color::White)
                Text(content: format!("Language: {}", trail.language), color: Color::White)
                Text(content: format!("Age Group: {}", trail.age_group), color: Color::White)
                Text(content: format!("Status: {}", trail.status), color: Color::White)
                Text(content: format!("Generated: {}", trail.generated_at), color: Color::White)
                Text(
                    content: format!("Tenant: {}", trail.tenant_id.as_ref().unwrap_or(&"N/A".to_string())),
                    color: Color::White
                )
                Text(content: format!("File: {}", trail.file_path), color: Color::Grey)
            }
        }
        .into_any(),
    );

    // Generation parameters section
    elements.push(
        element! {
            View(
                border_style: BorderStyle::Single,
                border_color: Color::Yellow,
                padding: 2,
                margin_bottom: 1
            ) {
                Text(content: "Generation Parameters", color: Color::Yellow, weight: Weight::Bold)
                Text(content: format!("Node Count: {}", trail.node_count), color: Color::White)
                Text(
                    content: format!("Tags: {}", trail.tags.join(", ")),
                    color: Color::White
                )
            }
        }
        .into_any(),
    );

    // Story structure section
    elements.push(render_story_structure_section(&props.full_data));

    // Execution trace section
    elements.push(render_execution_trace_section(&props.full_data));

    // Help text
    elements.push(
        element! {
            View(margin_top: 1, padding: 1) {
                Text(
                    content: "Esc: Back to List",
                    color: Color::Grey
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
    .into_any()
}

/// Render story structure section wrapper
fn render_story_structure_section(full_data: &Option<GenerationResponse>) -> AnyElement<'static> {
    match full_data {
        Some(data) => {
            let mut elements: Vec<AnyElement> = Vec::new();
            elements.push(
                element! {
                    Text(content: "Story Structure (DAG)", color: Color::Magenta, weight: Weight::Bold)
                }
                .into_any(),
            );
            elements.push(render_dag_tree(data));

            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Magenta,
                    padding: 2,
                    margin_bottom: 1
                ) {
                    #(elements.into_iter())
                }
            }.into_any()
        }
        None => {
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Magenta,
                    padding: 2,
                    margin_bottom: 1
                ) {
                    Text(content: "Story Structure (DAG)", color: Color::Magenta, weight: Weight::Bold)
                    Text(
                        content: "Loading full trail data...",
                        color: Color::Grey
                    )
                }
            }.into_any()
        }
    }
}

/// Render execution trace section wrapper
fn render_execution_trace_section(full_data: &Option<GenerationResponse>) -> AnyElement<'static> {
    match full_data {
        Some(data) => {
            let mut elements: Vec<AnyElement> = Vec::new();
            elements.push(
                element! {
                    Text(content: "Execution Trace", color: Color::Blue, weight: Weight::Bold)
                }
                .into_any(),
            );
            elements.push(render_execution_trace(data));

            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Blue,
                    padding: 2,
                    margin_bottom: 1
                ) {
                    #(elements.into_iter())
                }
            }.into_any()
        }
        None => {
            element! {
                View(
                    border_style: BorderStyle::Single,
                    border_color: Color::Blue,
                    padding: 2,
                    margin_bottom: 1
                ) {
                    Text(content: "Execution Trace", color: Color::Blue, weight: Weight::Bold)
                    Text(
                        content: "Loading execution trace...",
                        color: Color::Grey
                    )
                }
            }.into_any()
        }
    }
}

/// Render the DAG as ASCII tree (simple version for MVP)
fn render_dag_tree(generation_response: &GenerationResponse) -> AnyElement<'static> {
    // For MVP, display trail steps count
    // Full DAG visualization requires the metadata to contain DAG structure
    let trail_steps = match &generation_response.trail_steps {
        Some(steps) => steps,
        None => {
            return element! {
                Text(content: "No trail steps available", color: Color::Grey)
            }
            .into_any();
        }
    };

    let mut lines: Vec<AnyElement> = Vec::new();

    // Add summary
    lines.push(
        element! {
            Text(
                content: format!("Trail Steps: {}", trail_steps.len()),
                color: Color::White
            )
        }
        .into_any(),
    );

    // Display first few steps
    for (idx, step) in trail_steps.iter().take(5).enumerate() {
        let title = step.title.as_ref().map(|s| s.as_str()).unwrap_or("Untitled");
        lines.push(
            element! {
                Text(
                    content: format!("  {}. {} (Order: {})", idx + 1, title, step.step_order),
                    color: Color::White
                )
            }
            .into_any(),
        );
    }

    if trail_steps.len() > 5 {
        lines.push(
            element! {
                Text(
                    content: format!("  ... and {} more steps", trail_steps.len() - 5),
                    color: Color::Grey
                )
            }
            .into_any(),
        );
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(lines.into_iter())
        }
    }
    .into_any()
}

/// Render execution trace summary
fn render_execution_trace(generation_response: &GenerationResponse) -> AnyElement<'static> {
    let trace = match &generation_response.execution_trace {
        Some(t) => t,
        None => {
            return element! {
                Text(content: "No execution trace available", color: Color::Grey)
            }
            .into_any();
        }
    };

    let mut lines: Vec<AnyElement> = Vec::new();

    // Add summary
    lines.push(
        element! {
            Text(
                content: format!(
                    "Request ID: {} | Total Steps: {} | Duration: {}ms",
                    trace.request_id,
                    trace.service_invocations.len(),
                    trace.total_duration_ms
                ),
                color: Color::White
            )
        }
        .into_any(),
    );

    let events_count = trace.events_published.as_ref().map(|v| v.len()).unwrap_or(0);
    lines.push(
        element! {
            Text(
                content: format!(
                    "Phases Completed: {} | Events Published: {}",
                    trace.phases_completed.len(),
                    events_count
                ),
                color: Color::White
            )
        }
        .into_any(),
    );

    // List service invocations (show first 5)
    let max_display = 5;
    for (idx, invocation) in trace
        .service_invocations
        .iter()
        .take(max_display)
        .enumerate()
    {
        let service_name = &invocation.service_name;
        let tool_name = &invocation.tool_name;
        let status_color = if invocation.success {
            Color::Green
        } else {
            Color::Red
        };

        lines.push(
            element! {
                Text(
                    content: format!(
                        "{}. {} [{}] - {} ({}ms)",
                        idx + 1,
                        service_name,
                        tool_name,
                        if invocation.success { "Success" } else { "Failed" },
                        invocation.duration_ms
                    ),
                    color: status_color
                )
            }
            .into_any(),
        );
    }

    if trace.service_invocations.len() > max_display {
        lines.push(
            element! {
                Text(
                    content: format!("... and {} more steps", trace.service_invocations.len() - max_display),
                    color: Color::Grey
                )
            }
            .into_any(),
        );
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(lines.into_iter())
        }
    }
    .into_any()
}
