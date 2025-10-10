# HTTP Tests for Remote LLM Providers

This directory contains HTTP test files for verifying API connectivity and listing available models from different LLM providers.

## Prerequisites

### Environment Variables

Make sure your `.env` file in the parent directory contains the required API keys:

```bash
# OpenAI
LLM_OPENAI_API_KEY=sk-...

# Anthropic
LLM_ANTHROPIC_API_KEY=sk-ant-...

# Google Gemini
LLM_GOOGLE_API_KEY=AIza...
```

### Recommended Tools

These `.http` files work with:
- **VS Code**: Install the [REST Client extension](https://marketplace.visualstudio.com/items?itemName=humao.rest-client)
- **IntelliJ IDEA / WebStorm**: Built-in HTTP Client support
- **curl**: Files include curl equivalents in comments

## Usage

### VS Code with REST Client

1. Install REST Client extension
2. Open any `.http` file
3. Click "Send Request" above any `###` separator
4. View response in a new panel

### IntelliJ IDEA / WebStorm

1. Open any `.http` file
2. Click the green play button next to any request
3. View response in the Run panel

### Manual curl Commands

#### Google Gemini - List Models
```bash
curl "https://generativelanguage.googleapis.com/v1beta/models?key=$LLM_GOOGLE_API_KEY"
```

#### OpenAI - List Models
```bash
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $LLM_OPENAI_API_KEY"
```

## Files

### `google-list-models.http`
- Lists all available Google Gemini models
- Shows model capabilities and supported generation methods
- Includes specific model detail endpoints

**Available Models:**
- `gemini-1.5-flash-latest` - Fast, efficient
- `gemini-1.5-pro-latest` - Most capable
- `gemini-2.0-flash-exp` - Experimental

### `openai-list-models.http`
- Lists all OpenAI models available in your account
- Includes fine-tuned models if any
- Shows model details and permissions

### `anthropic-models.http`
- Documents known Anthropic Claude models
- Note: Anthropic doesn't have a public list models endpoint
- Includes test request to verify API connectivity

**Known Models:**
- `claude-3-5-sonnet-20241022` - Latest, best for complex tasks
- `claude-3-opus-20240229` - Most capable
- `claude-3-sonnet-20240229` - Balanced
- `claude-3-haiku-20240307` - Fastest

## Troubleshooting

### "Missing API Key" Error
- Verify `.env` file exists in parent directory
- Check API key variable names match exactly
- Restart your IDE/editor to reload environment

### "404 Not Found" for Gemini
- Ensure you're using model names with `-latest` suffix
- Google's v1beta API requires specific model naming

### "401 Unauthorized"
- Verify API key is valid and not expired
- Check API key has proper permissions
- For Google: Ensure Generative Language API is enabled in your project

## Expected Responses

### Google Gemini
```json
{
  "models": [
    {
      "name": "models/gemini-1.5-flash-latest",
      "displayName": "Gemini 1.5 Flash",
      "supportedGenerationMethods": ["generateContent", "streamGenerateContent"]
    }
  ]
}
```

### OpenAI
```json
{
  "data": [
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1234567890,
      "owned_by": "openai"
    }
  ]
}
```

### Anthropic
```json
{
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "I am Claude..."
    }
  ]
}
```

## Integration with TaleTrail

These tests verify the models configured in:
- `prompt-helper/.env` - LLM provider configuration
- `story-generator/.env` - Story generation LLM settings

Use these tests to:
1. Verify API connectivity before running services
2. Discover available models for your API keys
3. Test new model IDs before updating configuration
4. Debug authentication issues
