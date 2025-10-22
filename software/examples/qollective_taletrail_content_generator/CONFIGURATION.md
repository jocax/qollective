# TaleTrail DAG Configuration Guide

> Complete guide to configuring story structure in TaleTrail Content Generator

## Table of Contents

- [DAG Configuration Overview](#dag-configuration-overview)
- [Orchestrator Configuration (config.toml)](#orchestrator-configuration-configtoml)
- [Request-Level Configuration](#request-level-configuration)
  - [Option A: Story Structure Presets](#option-a-story-structure-presets-tier-1)
  - [Option B: Custom DAG Configuration](#option-b-custom-dag-configuration-tier-2)
- [Validation Rules](#validation-rules)
- [Migration Guide](#migration-guide)
- [Troubleshooting](#troubleshooting)

## DAG Configuration Overview

TaleTrail generates interactive stories as **Directed Acyclic Graphs (DAGs)** where each node represents a story segment and edges represent reader choices. The DAG structure determines the complexity, branching patterns, and convergence behavior of generated stories.

### What DAG Configuration Controls

The DAG configuration specifies:

- **Node Count**: Total number of story segments (4-100 nodes)
- **Convergence Pattern**: How story branches converge back together
- **Convergence Point Ratio**: Position in the story where convergence occurs (0.0-1.0)
- **Max Depth**: Maximum depth of the DAG tree structure (3-20 levels)
- **Branching Factor**: Number of choices at each decision point (2-4 choices)

### Two-Tier Configuration Model

TaleTrail uses a **two-tier model** that balances simplicity with advanced control:

**Tier 1: Story Structure Presets (Simple)**
- Predefined configurations for common storytelling patterns
- Use `story_structure` field with preset name
- No manual parameter tuning required
- Ideal for 90% of educational content use cases

**Tier 2: Custom DAG Configuration (Advanced)**
- Complete control over all 5 DAG parameters
- Use `dag_config` object with explicit values
- Fine-grained control for specialized scenarios
- Ideal for experimentation and advanced use cases

**Design Philosophy:**
- Most users should use presets for consistency and simplicity
- Advanced users can override with custom configurations
- Three-tier priority system ensures predictable behavior
- Validation ensures all configurations produce valid DAGs

## Orchestrator Configuration (config.toml)

The orchestrator maintains default DAG configuration in `orchestrator/config.toml`. These defaults serve as **Priority 3** fallback when neither preset nor custom config is provided in requests.

### DAG Section

```toml
[dag]
# Default DAG structure configuration (Priority 3: Lowest)
# Used when request contains neither story_structure nor dag_config

# Total number of nodes in story DAG (4-100)
node_count = 12

# Pattern for how story branches converge
# Valid values: "SingleConvergence", "MultipleConvergence", "EndOnly",
#               "PureBranching", "ParallelPaths"
convergence_pattern = "SingleConvergence"

# Position of convergence as ratio (0.0-1.0, where 0.5 = midpoint)
# Required for SingleConvergence, MultipleConvergence, EndOnly
# Must be omitted for PureBranching and ParallelPaths
convergence_point_ratio = 0.5

# Maximum depth of DAG tree structure (3-20)
max_depth = 8

# Number of choices per decision node (2-4)
branching_factor = 2
```

### Configuration Fields

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `node_count` | integer | 12 | 4-100 | Total story nodes |
| `convergence_pattern` | string | "SingleConvergence" | See patterns | Convergence behavior |
| `convergence_point_ratio` | float | 0.5 | 0.0-1.0 | Convergence position |
| `max_depth` | integer | 8 | 3-20 | Maximum tree depth |
| `branching_factor` | integer | 2 | 2-4 | Choices per node |

### Convergence Patterns

| Pattern | Description | Requires Ratio? | Use Case |
|---------|-------------|-----------------|----------|
| `SingleConvergence` | One major convergence point | Yes | Linear stories with one turning point |
| `MultipleConvergence` | Multiple convergences at intervals | Yes | Adventure stories with multiple junctions |
| `EndOnly` | Converges only at climax | Yes | Epic stories with complex branching |
| `PureBranching` | No convergence, multiple endings | No | Choose-your-own-adventure books |
| `ParallelPaths` | Parallel tracks that don't converge | No | Multi-perspective narratives |

### When Orchestrator Defaults Are Used

The orchestrator defaults serve as **Priority 3 (Lowest)** fallback:

```json
{
  "theme": "Space Exploration",
  "age_group": "9-11",
  "language": "en"
}
```

In this request:
- No `story_structure` preset provided (Priority 1)
- No `dag_config` object provided (Priority 2)
- **Orchestrator defaults from config.toml will be used** (Priority 3)

## Request-Level Configuration

Requests can override orchestrator defaults using either presets (Priority 1) or custom configuration (Priority 2).

### Option A: Story Structure Presets (Tier 1)

**Priority: 1 (Highest)**

Story structure presets provide predefined DAG configurations optimized for common storytelling patterns. Use the `story_structure` field with one of 4 preset names.

#### Available Presets

| Preset Name | Nodes | Pattern | Convergence | Max Depth | Branching | Optimized For |
|-------------|-------|---------|-------------|-----------|-----------|---------------|
| `guided` | 12 | SingleConvergence | 50% (node 6) | 8 | 2 | Linear educational stories |
| `adventure` | 16 | MultipleConvergence | 60% intervals | 10 | 2 | Branching adventure stories |
| `epic` | 24 | EndOnly | 90% (node ~21) | 12 | 2 | Complex epic narratives |
| `choose_your_path` | 16 | PureBranching | None | 10 | 3 | Multiple ending stories |

#### Preset Details

**Guided Preset**
```json
{
  "story_structure": "guided"
}
```
- **Use Case**: Linear story with occasional choices and single convergence
- **Configuration**: 12 nodes, depth 8, branching factor 2
- **Convergence**: SingleConvergence at 50% through story
- **Ideal For**: Younger age groups (6-8, 9-11), educational focus
- **Example**: "Ocean exploration with one major decision point"

**Adventure Preset**
```json
{
  "story_structure": "adventure"
}
```
- **Use Case**: Adventure story with multiple convergence points
- **Configuration**: 16 nodes, depth 10, branching factor 2
- **Convergence**: MultipleConvergence at 60% intervals
- **Ideal For**: Middle age groups (9-11, 12-14), exploration themes
- **Example**: "Treasure hunt with multiple paths that reunite"

**Epic Preset**
```json
{
  "story_structure": "epic"
}
```
- **Use Case**: Epic story with complex branching that converges at the end
- **Configuration**: 24 nodes, depth 12, branching factor 2
- **Convergence**: EndOnly at 90% (node ~21)
- **Ideal For**: Older age groups (12-14, 15-17), complex narratives
- **Example**: "Quest narrative with many paths to final confrontation"

**Choose Your Path Preset**
```json
{
  "story_structure": "choose_your_path"
}
```
- **Use Case**: Pure branching tree with multiple endings
- **Configuration**: 16 nodes, depth 10, branching factor 3
- **Convergence**: None (PureBranching)
- **Ideal For**: All age groups, reader agency emphasis
- **Example**: "Mystery with 5-6 different possible endings"

#### JSON Structure

```json
{
  "theme": "Ocean Adventure with Marine Biology",
  "age_group": "9-11",
  "language": "en",
  "educational_goals": ["ocean ecosystem", "marine life", "conservation"],
  "vocabulary_level": "intermediate",
  "required_elements": ["teamwork message", "ocean conservation"],
  "tags": ["ocean", "adventure", "educational"],
  "story_structure": "guided"
}
```

**Note:** Preset names are case-insensitive (`"guided"`, `"GUIDED"`, and `"Guided"` are all valid).

#### When to Use Presets

Use presets when:
- Building standard educational content
- Prototyping story structures quickly
- Maintaining consistency across multiple stories
- Working with content creators unfamiliar with DAG parameters

### Option B: Custom DAG Configuration (Tier 2)

**Priority: 2 (Middle)**

Custom DAG configuration provides complete control over all DAG parameters. Use the `dag_config` object with explicit values for all 5 parameters.

#### Parameters

| Parameter | Type | Required | Range | Description |
|-----------|------|----------|-------|-------------|
| `node_count` | integer | Yes | 4-100 | Total nodes in story DAG |
| `convergence_pattern` | string enum | Yes | 5 patterns | How branches converge |
| `convergence_point_ratio` | float | Conditional | 0.0-1.0 | Position of convergence |
| `max_depth` | integer | Yes | 3-20 | Maximum depth of tree |
| `branching_factor` | integer | Yes | 2-4 | Choices per decision node |

#### Parameter Details

**node_count**
- **Type**: Integer
- **Range**: 4-100
- **Default**: None (required)
- **Description**: Total number of story segments in the DAG
- **Constraints**:
  - Minimum 4: Required for meaningful branching structure
  - Maximum 100: Performance and readability limit
  - Should be even for balanced convergence calculations
- **Examples**:
  - `8`: Short story with minimal branching
  - `16`: Standard interactive story
  - `32`: Long-form narrative with complex structure
  - `100`: Maximum complexity for experimental content

**convergence_pattern**
- **Type**: String enum
- **Range**: `"SingleConvergence"`, `"MultipleConvergence"`, `"EndOnly"`, `"PureBranching"`, `"ParallelPaths"`
- **Default**: None (required)
- **Description**: Pattern for how story branches converge
- **Patterns**:
  - `"SingleConvergence"`: One major convergence point (requires ratio)
  - `"MultipleConvergence"`: Multiple convergences at intervals (requires ratio)
  - `"EndOnly"`: Converges only at story climax (requires ratio)
  - `"PureBranching"`: No convergence, multiple endings (no ratio)
  - `"ParallelPaths"`: Parallel story tracks (no ratio)

**convergence_point_ratio**
- **Type**: Float (optional)
- **Range**: 0.0-1.0 (0.5 = midpoint)
- **Default**: None
- **Description**: Position in story where convergence occurs
- **Constraints**:
  - **Required** for: `SingleConvergence`, `MultipleConvergence`, `EndOnly`
  - **Must be omitted** for: `PureBranching`, `ParallelPaths`
  - Must be between 0.0 (start) and 1.0 (end)
- **Examples**:
  - `0.33`: Converge at 1/3 through story
  - `0.5`: Converge at midpoint
  - `0.75`: Converge near the end
  - `0.9`: Converge just before climax

**max_depth**
- **Type**: Integer
- **Range**: 3-20
- **Default**: None (required)
- **Description**: Maximum depth of the DAG tree structure
- **Constraints**:
  - Minimum 3: Required for branching structure
  - Maximum 20: Cognitive load and complexity limit
- **Guidelines**:
  - Depth 3-5: Simple stories (2-3 choice layers)
  - Depth 6-10: Standard stories (4-6 choice layers)
  - Depth 11-15: Complex stories (7-10 choice layers)
  - Depth 16-20: Highly complex stories (experimental)

**branching_factor**
- **Type**: Integer
- **Range**: 2-4
- **Default**: None (required)
- **Description**: Number of choices at each decision node
- **Constraints**:
  - Minimum 2: Binary choices (yes/no, left/right)
  - Maximum 4: Cognitive load limit for readers
- **Guidelines**:
  - Factor 2: Binary choices, simplest branching
  - Factor 3: Common for "choose your own adventure"
  - Factor 4: Maximum complexity, use sparingly

#### JSON Structure

```json
{
  "theme": "Medieval Quest for the Sacred Artifact",
  "age_group": "12-14",
  "language": "en",
  "educational_goals": ["medieval history", "ethics", "strategy"],
  "vocabulary_level": "intermediate",
  "required_elements": ["honor code", "strategic thinking"],
  "tags": ["medieval", "quest", "strategy"],
  "dag_config": {
    "node_count": 20,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.33,
    "max_depth": 15,
    "branching_factor": 2
  }
}
```

#### When to Use Custom Configuration

Use custom DAG configuration when:
- Experimenting with new story structures
- Building specialized educational content with unique requirements
- Creating content that doesn't fit predefined presets
- Fine-tuning convergence behavior for specific narratives
- Developing new preset candidates for testing

#### Example Configurations

**Short Story with Early Convergence:**
```json
{
  "dag_config": {
    "node_count": 8,
    "convergence_pattern": "SingleConvergence",
    "convergence_point_ratio": 0.25,
    "max_depth": 5,
    "branching_factor": 2
  }
}
```

**Complex Multi-Path Epic:**
```json
{
  "dag_config": {
    "node_count": 40,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.66,
    "max_depth": 18,
    "branching_factor": 3
  }
}
```

**Pure Branching with Maximum Complexity:**
```json
{
  "dag_config": {
    "node_count": 32,
    "convergence_pattern": "PureBranching",
    "convergence_point_ratio": null,
    "max_depth": 12,
    "branching_factor": 4
  }
}
```

## Configuration Priority System

TaleTrail uses a three-tier priority system to determine which DAG configuration to use:

### Priority Hierarchy

| Priority | Source | Field | When Used |
|----------|--------|-------|-----------|
| 1 (Highest) | Request JSON | `story_structure` | Preset name provided |
| 2 (Middle) | Request JSON | `dag_config` | Custom config object provided |
| 3 (Lowest) | config.toml | `[dag]` section | Neither preset nor config provided |

### Priority Rules

**Rule 1: Preset Always Wins**

If `story_structure` is provided, it takes priority over everything:

```json
{
  "theme": "Space Exploration",
  "age_group": "9-11",
  "language": "en",
  "story_structure": "guided",
  "dag_config": {
    "node_count": 20,
    "convergence_pattern": "Epic"
  }
}
```

**Result**: `guided` preset configuration used (12 nodes, SingleConvergence).
**Behavior**: `dag_config` is logged and ignored with warning message.

**Rule 2: Custom Config When No Preset**

If `dag_config` is provided but `story_structure` is not:

```json
{
  "theme": "Mystery Investigation",
  "age_group": "12-14",
  "language": "en",
  "dag_config": {
    "node_count": 18,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.5,
    "max_depth": 12,
    "branching_factor": 3
  }
}
```

**Result**: Custom configuration used exactly as specified.
**Behavior**: All parameters validated before use.

**Rule 3: Orchestrator Defaults as Fallback**

If neither `story_structure` nor `dag_config` is provided:

```json
{
  "theme": "Historical Adventure",
  "age_group": "9-11",
  "language": "en"
}
```

**Result**: Configuration from `orchestrator/config.toml` `[dag]` section used.
**Behavior**: Default values applied silently.

### Logging Behavior

The orchestrator logs configuration source for transparency:

```
INFO: Using story_structure preset 'guided' for DAG configuration
WARN: Both story_structure and dag_config provided. Using preset 'guided', ignoring dag_config.
INFO: Using custom dag_config: node_count=20, convergence_pattern=MultipleConvergence
INFO: Using orchestrator default DAG configuration from config.toml
```

## Validation Rules

TaleTrail validates all DAG configurations before generation to ensure valid story structures.

### Parameter Validation

**node_count Validation**
- **Rule**: Must be integer between 4 and 100
- **Rationale**:
  - Minimum 4: Required for meaningful branching structure
  - Maximum 100: Performance and readability constraints
- **Error Message**: `"node_count must be between 4 and 100, got {value}"`

**convergence_pattern Validation**
- **Rule**: Must be one of 5 valid enum values
- **Valid Values**:
  - `"SingleConvergence"`
  - `"MultipleConvergence"`
  - `"EndOnly"`
  - `"PureBranching"`
  - `"ParallelPaths"`
- **Error Message**: `"Invalid convergence_pattern '{value}'. Valid options: SingleConvergence, MultipleConvergence, EndOnly, PureBranching, ParallelPaths"`

**convergence_point_ratio Validation**
- **Rule 1**: Required for `SingleConvergence`, `MultipleConvergence`, `EndOnly`
- **Rule 2**: Must be omitted for `PureBranching` and `ParallelPaths`
- **Rule 3**: When provided, must be between 0.0 and 1.0
- **Error Messages**:
  - `"convergence_point_ratio required for {pattern} but not provided"`
  - `"convergence_point_ratio must be omitted for {pattern}"`
  - `"convergence_point_ratio must be between 0.0 and 1.0, got {value}"`

**max_depth Validation**
- **Rule**: Must be integer between 3 and 20
- **Rationale**:
  - Minimum 3: Required for branching structure
  - Maximum 20: Cognitive complexity limit
- **Error Message**: `"max_depth must be between 3 and 20, got {value}"`

**branching_factor Validation**
- **Rule**: Must be integer between 2 and 4
- **Rationale**:
  - Minimum 2: Binary choices minimum
  - Maximum 4: Reader cognitive load limit
- **Error Message**: `"branching_factor must be between 2 and 4, got {value}"`

### Preset Validation

**Preset Name Validation**
- **Rule**: Must be one of 4 valid preset names (case-insensitive)
- **Valid Names**: `"guided"`, `"adventure"`, `"epic"`, `"choose_your_path"`
- **Error Message**: `"Unknown story_structure preset: '{name}'. Valid options: guided, adventure, epic, choose_your_path"`

### Validation Timing

Validation occurs at multiple stages:

1. **Request Parsing**: JSON schema validation ensures correct types
2. **Configuration Resolution**: Preset lookup and custom config parsing
3. **Pre-Generation**: Complete DAG config validation before generation starts
4. **Early Failure**: Invalid configurations fail fast with clear error messages

### Validation Examples

**Valid Configuration:**
```json
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.6,
    "max_depth": 10,
    "branching_factor": 2
  }
}
```
✅ All parameters in valid ranges and consistent with pattern.

**Invalid: node_count Out of Range**
```json
{
  "dag_config": {
    "node_count": 150,
    "convergence_pattern": "SingleConvergence",
    "convergence_point_ratio": 0.5,
    "max_depth": 10,
    "branching_factor": 2
  }
}
```
❌ Error: `"node_count must be between 4 and 100, got 150"`

**Invalid: Missing convergence_point_ratio**
```json
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "SingleConvergence",
    "max_depth": 10,
    "branching_factor": 2
  }
}
```
❌ Error: `"convergence_point_ratio required for SingleConvergence but not provided"`

**Invalid: Unnecessary convergence_point_ratio**
```json
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "PureBranching",
    "convergence_point_ratio": 0.5,
    "max_depth": 10,
    "branching_factor": 3
  }
}
```
❌ Error: `"convergence_point_ratio must be omitted for PureBranching"`

**Invalid: Unknown Preset**
```json
{
  "story_structure": "super_epic"
}
```
❌ Error: `"Unknown story_structure preset: 'super_epic'. Valid options: guided, adventure, epic, choose_your_path"`

## Migration Guide

### Upgrading from Hardcoded Defaults

If you're upgrading from a version with hardcoded DAG defaults:

**Step 1: Identify Current Defaults**

Check your orchestrator code for hardcoded values:
```rust
// Old hardcoded approach
let node_count = 12;
let convergence_pattern = ConvergencePattern::SingleConvergence;
```

**Step 2: Add [dag] Section to config.toml**

Create orchestrator defaults:
```toml
[dag]
node_count = 12
convergence_pattern = "SingleConvergence"
convergence_point_ratio = 0.5
max_depth = 8
branching_factor = 2
```

**Step 3: Use Presets for Common Patterns**

Replace hardcoded patterns with presets:
```json
{
  "story_structure": "guided"
}
```

**Step 4: Test Configuration Priority**

Verify the three-tier priority system:
1. Test with preset only
2. Test with custom config only
3. Test with neither (defaults)
4. Test with both (preset should win)

### Backward Compatibility

The two-tier model maintains backward compatibility:

**Legacy Behavior (Pre-Phase 4):**
```json
{
  "theme": "Adventure",
  "age_group": "9-11",
  "language": "en",
  "node_count": 16
}
```
✅ Still works: `node_count` in top-level request overrides orchestrator default.

**New Behavior (Phase 4+):**
```json
{
  "theme": "Adventure",
  "age_group": "9-11",
  "language": "en",
  "story_structure": "adventure"
}
```
✅ Recommended: Use preset for consistent configuration.

### Testing Configuration Changes

After migration, test all configuration sources:

```bash
# Test with guided preset
cargo run -p nats-cli -- send --template templates/orchestrator/request_guided.json

# Test with adventure preset
cargo run -p nats-cli -- send --template templates/orchestrator/request_adventure.json

# Test with epic preset
cargo run -p nats-cli -- send --template templates/orchestrator/request_epic.json

# Test with choose_your_path preset
cargo run -p nats-cli -- send --template templates/orchestrator/request_choose_your_path.json

# Test with custom DAG config
cargo run -p nats-cli -- send --template templates/orchestrator/request_custom_dag.json

# Test with neither (uses orchestrator defaults)
cargo run -p nats-cli -- send --template templates/orchestrator/generate_story.json
```

## Troubleshooting

### Invalid Preset Name

**Problem**: Request fails with "Unknown story_structure preset" error

**Cause**: Typo in preset name or unsupported preset value

**Solution**:
```json
// ❌ Wrong
{
  "story_structure": "super_epic"
}

// ✅ Correct
{
  "story_structure": "epic"
}
```

**Valid Options**: `"guided"`, `"adventure"`, `"epic"`, `"choose_your_path"`

### Missing convergence_point_ratio

**Problem**: Validation error: "convergence_point_ratio required but not provided"

**Cause**: Pattern requires ratio but it's missing

**Solution**:
```json
// ❌ Wrong - Missing ratio for SingleConvergence
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "SingleConvergence",
    "max_depth": 10,
    "branching_factor": 2
  }
}

// ✅ Correct - Ratio provided
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "SingleConvergence",
    "convergence_point_ratio": 0.5,
    "max_depth": 10,
    "branching_factor": 2
  }
}
```

**Rule**: `convergence_point_ratio` required for `SingleConvergence`, `MultipleConvergence`, and `EndOnly`.

### Unnecessary convergence_point_ratio

**Problem**: Validation error: "convergence_point_ratio must be omitted"

**Cause**: Ratio provided for pattern that doesn't use it

**Solution**:
```json
// ❌ Wrong - Ratio not needed for PureBranching
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "PureBranching",
    "convergence_point_ratio": 0.5,
    "max_depth": 10,
    "branching_factor": 3
  }
}

// ✅ Correct - Ratio omitted
{
  "dag_config": {
    "node_count": 16,
    "convergence_pattern": "PureBranching",
    "max_depth": 10,
    "branching_factor": 3
  }
}
```

**Rule**: Omit `convergence_point_ratio` for `PureBranching` and `ParallelPaths`.

### Node Count Out of Range

**Problem**: Validation error: "node_count must be between 4 and 100"

**Cause**: Node count below minimum (4) or above maximum (100)

**Solution**:
```json
// ❌ Wrong - Too few nodes
{
  "dag_config": {
    "node_count": 2
  }
}

// ✅ Correct - Valid range
{
  "dag_config": {
    "node_count": 16
  }
}
```

**Valid Range**: 4-100 nodes

### Both Preset and Custom Config Provided

**Problem**: Warning logged: "Both story_structure and dag_config provided"

**Cause**: Request contains both configuration options

**Behavior**: Preset wins (Priority 1), custom config ignored

**Solution**:

If preset is intended:
```json
// ✅ Use preset only
{
  "story_structure": "guided"
}
```

If custom config is intended:
```json
// ✅ Use custom config only
{
  "dag_config": {
    "node_count": 20,
    "convergence_pattern": "MultipleConvergence",
    "convergence_point_ratio": 0.33,
    "max_depth": 15,
    "branching_factor": 2
  }
}
```

**Note**: Providing both is not an error, but the custom config will be ignored.

### Configuration Not Applied

**Problem**: Generated story doesn't match expected configuration

**Cause**: Configuration priority not understood

**Diagnostic Steps**:

1. Check orchestrator logs for configuration source:
   ```
   INFO: Using story_structure preset 'guided' for DAG configuration
   INFO: Using custom dag_config: node_count=20
   INFO: Using orchestrator default DAG configuration
   ```

2. Verify request JSON contains intended configuration
3. Check orchestrator `config.toml` for default values
4. Understand three-tier priority system

**Solution**: Ensure configuration is provided at correct priority level.

### Invalid Convergence Pattern

**Problem**: Validation error: "Invalid convergence_pattern"

**Cause**: Typo in pattern name or invalid value

**Solution**:
```json
// ❌ Wrong - Invalid pattern
{
  "dag_config": {
    "convergence_pattern": "NoConvergence"
  }
}

// ✅ Correct - Valid pattern
{
  "dag_config": {
    "convergence_pattern": "PureBranching"
  }
}
```

**Valid Patterns**:
- `"SingleConvergence"`
- `"MultipleConvergence"`
- `"EndOnly"`
- `"PureBranching"`
- `"ParallelPaths"`

---

## Additional Resources

- **README.md**: Quick reference for DAG configuration
- **schemas/taletrail-content-generator.json**: JSON Schema definitions
- **shared-types-generated/src/presets.rs**: Preset implementation
- **shared-types-generated/src/extensions/generation_request.rs**: Configuration resolution logic
- **orchestrator/config.toml**: Default DAG configuration

## Version History

- **Phase 4.8**: Two-tier model implemented with presets and custom config
- **Phase 4.7**: Validation rules and priority system
- **Phase 4.6**: Orchestrator defaults in config.toml
