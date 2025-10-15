# MCP Request Templates

This directory contains JSON templates for MCP (Model Context Protocol) tool call requests. Templates are organized by MCP server name.

## Directory Structure

```
templates/
├── README.md                              # This file
└── <server-name>/                         # One directory per MCP server
    ├── <template-name>.json              # Request template files
    └── ...
```

## Template Format

Each template is a JSON file with the following structure:

```json
{
  "tool_name": "<mcp-tool-name>",
  "arguments": {
    "<arg1>": "<value1>",
    "<arg2>": "<value2>",
    ...
  }
}
```

### Fields

- **tool_name** (required): The name of the MCP tool to call
- **arguments** (optional): JSON object containing tool arguments

### Example Template

```json
{
  "tool_name": "generate_story_prompts",
  "arguments": {
    "theme": "Space Adventure",
    "age_group": "6-8",
    "language": "en",
    "educational_goals": ["science", "creativity"],
    "vocabulary_level": "basic"
  }
}
```

## Available Templates

### prompt-helper Server

Templates for the `prompt-helper` MCP server:

#### 1. generate_story_prompts.json

Generate story prompts with educational goals and vocabulary level.

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_story_prompts.json
```

**Arguments:**
- `theme` (string): Story theme (e.g., "Space Adventure")
- `age_group` (string): Target age group (e.g., "6-8")
- `language` (string): Language code (e.g., "en")
- `educational_goals` (array): List of educational objectives
- `vocabulary_level` (string): Vocabulary complexity level

#### 2. generate_validation_prompts.json

Generate validation prompts for content quality control.

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_validation_prompts.json
```

**Arguments:**
- `age_group` (string): Target age group for validation
- `language` (string): Language code
- `content_type` (string): Type of content to validate

#### 3. generate_constraint_prompts.json

Generate constraint prompts for content generation boundaries.

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/generate_constraint_prompts.json
```

**Arguments:**
- `vocabulary_level` (string): Required vocabulary level
- `language` (string): Language code
- `required_elements` (array): Must-have content elements

#### 4. get_model_for_language.json

Get the recommended LLM model for a specific language.

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/get_model_for_language.json
```

**Arguments:**
- `language` (string): Language code (e.g., "en", "es", "fr")

### story-generator Server

Templates for the `story-generator` MCP server:

#### 1. generate_structure.json

Generate the DAG (Directed Acyclic Graph) structure for an interactive story with nodes, edges, and convergence points.

```bash
nats-cli send --subject mcp.story.generate \
  --template templates/story-generator/generate_structure.json
```

**Arguments:**
- `generation_request` (object): Complete story generation request
  - `theme` (string): Story theme (e.g., "Space Adventure")
  - `age_group` (string): Target age group (e.g., "6-8")
  - `language` (string): Language code (e.g., "en")
  - `node_count` (number): Number of story nodes to generate
  - `tenant_id` (number): Tenant identifier
  - `educational_goals` (array): List of educational objectives
  - `vocabulary_level` (string): Vocabulary complexity level (e.g., "basic", "intermediate")
  - `required_elements` (array): Must-have content elements
  - `prompt_packages` (object): LLM prompt configurations for each service

#### 2. generate_nodes.json

Generate actual story content for specific nodes in the DAG structure.

```bash
nats-cli send --subject mcp.story.generate \
  --template templates/story-generator/generate_nodes.json
```

**Arguments:**
- `dag` (object): Complete DAG structure with nodes and edges
- `node_ids` (array): List of node IDs to generate content for
- `generation_request` (object): Same as generate_structure.json

### constraint-enforcer Server

Templates for the `constraint-enforcer` MCP server:

#### 1. enforce_constraints.json

Enforce vocabulary, theme, and content constraints on a generated story node.

```bash
nats-cli send --subject mcp.constraint.enforce \
  --template templates/constraint-enforcer/enforce_constraints.json
```

**Arguments:**
- `content_node` (object): Story node to validate
  - `node_id` (string): Unique node identifier
  - `content` (string): Story text content
  - `choices` (array): List of choice options
  - `metadata` (object): Node metadata (word_count, reading_level, etc.)
- `generation_request` (object): Original generation request with constraints
  - `theme` (string): Story theme
  - `age_group` (string): Target age group
  - `language` (string): Language code
  - `node_count` (number): Total nodes in story
  - `tenant_id` (number): Tenant identifier
  - `educational_goals` (array): Educational objectives
  - `vocabulary_level` (string): Required vocabulary level
  - `required_elements` (array): Must-have content elements

#### 2. suggest_corrections.json

Generate specific correction suggestions for content that violates constraints.

```bash
nats-cli send --subject mcp.constraint.enforce \
  --template templates/constraint-enforcer/suggest_corrections.json
```

**Arguments:**
- `content_node` (object): Story node with constraint violations
  - `node_id` (string): Unique node identifier
  - `content` (string): Story text content (potentially violating constraints)
  - `choices` (array): List of choice options
  - `metadata` (object): Node metadata
- `generation_request` (object): Original generation request with constraints (same structure as enforce_constraints.json)

### quality-control Server

Templates for the `quality-control` MCP server:

#### 1. validate_content.json

Validate a single content node for age-appropriateness, safety, and educational value.

```bash
nats-cli send --subject mcp.quality.validate \
  --template templates/quality-control/validate_content.json
```

**Arguments:**
- `content_node` (object): Story node to validate
  - `node_id` (string): Unique node identifier
  - `content` (string): Story text content
  - `choices` (array): List of choice options
  - `metadata` (object): Node metadata (word_count, reading_level, etc.)
- `age_group` (string): Target age group (e.g., "6-8", "9-11")
- `educational_goals` (array): List of educational objectives to validate against

#### 2. batch_validate.json

Validate multiple content nodes efficiently in a single request.

```bash
nats-cli send --subject mcp.quality.validate \
  --template templates/quality-control/batch_validate.json
```

**Arguments:**
- `content_nodes` (array): List of story nodes to validate
  - Each node contains: `node_id`, `content`, `choices`, `metadata`
- `age_group` (string): Target age group
- `educational_goals` (array): Educational objectives to validate against

### orchestrator Server

Templates for the `orchestrator` MCP server:

#### 1. generate_story.json

Orchestrate the complete story generation pipeline, coordinating all services.

```bash
nats-cli send --subject mcp.orchestrator.request \
  --template templates/orchestrator/generate_story.json
```

**Arguments:**
- `generation_request` (object): Complete story generation request
  - `theme` (string): Story theme (e.g., "Ocean Adventure", "Space Adventure")
  - `age_group` (string): Target age group (e.g., "6-8", "9-11")
  - `language` (string): Language code (e.g., "en")
  - `node_count` (number): Number of story nodes to generate
  - `tenant_id` (number): Tenant identifier
  - `educational_goals` (array): List of educational objectives
  - `vocabulary_level` (string): Vocabulary complexity level (e.g., "basic", "intermediate", "advanced")
  - `required_elements` (array): Must-have content elements (e.g., "moral lesson about protecting ocean life", "factual information about marine ecosystems")

## Creating Custom Templates

To create a new template:

1. Navigate to the appropriate server directory (or create a new one)
2. Create a `.json` file with a descriptive name
3. Follow the template format shown above
4. Test your template:

```bash
nats-cli template list --server <server-name>
nats-cli send --subject <subject> --template templates/<server-name>/<template-name>.json
```

## Tips

- **Naming Convention**: Use descriptive names like `action_resource.json` (e.g., `generate_story_prompts.json`)
- **Validation**: Templates are validated when loaded - you'll get clear error messages if the format is incorrect
- **Arguments**: The `arguments` field can contain any valid JSON structure (objects, arrays, primitives)
- **No Arguments**: If a tool requires no arguments, you can omit the `arguments` field or set it to `null`

## Example: Template Without Arguments

```json
{
  "tool_name": "list_available_languages"
}
```

## Listing Templates

List all available templates:

```bash
nats-cli template list
```

List templates for a specific server:

```bash
nats-cli template list --server prompt-helper
```

## Environment-Specific Templates

You can create environment-specific template directories:

```
templates/
├── prompt-helper/
│   ├── dev/
│   │   └── test_with_mock_data.json
│   ├── staging/
│   │   └── test_with_staging_data.json
│   └── production/
│       └── production_request.json
```

Then reference them explicitly:

```bash
nats-cli send --subject mcp.prompt.helper \
  --template templates/prompt-helper/dev/test_with_mock_data.json
```

## Troubleshooting

### Template Not Found

Error: `Template not found: <path>`

**Solution**: Check that:
- The file exists at the specified path
- The path is correct (relative to where you're running the CLI)
- The file has a `.json` extension

### Invalid Template Format

Error: `Invalid template format: <details>`

**Solution**: Validate your JSON:
- Use a JSON validator (e.g., `jq . < template.json`)
- Ensure `tool_name` field is present
- Check for syntax errors (missing commas, brackets, quotes)

### Tool Execution Error

If the request is sent but the server returns an error, it's likely an issue with the arguments, not the template format.

**Solution**:
- Check the server's tool documentation for required/optional arguments
- Verify argument types match expectations
- Check logs for detailed error messages
