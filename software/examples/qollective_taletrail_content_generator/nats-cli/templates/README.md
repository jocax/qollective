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
