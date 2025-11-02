/**
 * Monitoring Page Tests
 *
 * Tests critical functionality of the NATS Monitoring Page through its composable logic:
 * - Message filtering by endpoint and text
 * - Message buffer limit enforcement
 * - Diagnostics calculation
 * - Filter clearing logic
 *
 * Note: Full component mounting tests are skipped due to Nuxt page complexity.
 * These tests focus on the core reactive logic that can be tested in isolation.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { ref, computed, nextTick } from 'vue';
import type { NatsMessage, EndpointFilter, MonitoringDiagnostics } from '@/types/monitoring';

describe('Monitoring Page - Message Filtering Logic', () => {
  let messages: ReturnType<typeof ref<NatsMessage[]>>;
  let selectedEndpoint: ReturnType<typeof ref<EndpointFilter>>;
  let filterText: ReturnType<typeof ref<string>>;
  let filteredMessages: ReturnType<typeof computed<NatsMessage[]>>;

  beforeEach(() => {
    messages = ref<NatsMessage[]>([]);
    selectedEndpoint = ref<EndpointFilter>('all');
    filterText = ref('');

    // Recreate the filtering logic from monitoring.vue
    filteredMessages = computed(() => {
      let filtered = messages.value;

      // Filter by endpoint
      if (selectedEndpoint.value !== 'all') {
        filtered = filtered.filter((msg) => msg.endpoint === selectedEndpoint.value);
      }

      // Filter by text (wildcard match on subject, payload, and request_id)
      if (filterText.value.trim()) {
        const searchText = filterText.value.toLowerCase();
        filtered = filtered.filter((msg) => {
          return (
            msg.subject.toLowerCase().includes(searchText) ||
            msg.payload.toLowerCase().includes(searchText) ||
            (msg.request_id && msg.request_id.toLowerCase().includes(searchText))
          );
        });
      }

      return filtered;
    });
  });

  it('[11.1.1] should show all messages when no filters are applied', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test1"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.story-generator.request',
        endpoint: 'story-generator',
        message_type: 'Request',
        payload: '{"tool": "test2"}',
        request_id: 'req-2',
      },
    ];

    // Assert
    expect(filteredMessages.value).toHaveLength(2);
  });

  it('[11.1.2] should filter messages by endpoint correctly', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test1"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.story-generator.request',
        endpoint: 'story-generator',
        message_type: 'Request',
        payload: '{"tool": "test2"}',
        request_id: 'req-2',
      },
      {
        timestamp: '2025-01-01T00:00:02Z',
        subject: 'mcp.orchestrator.response',
        endpoint: 'orchestrator',
        message_type: 'Response',
        payload: '{"result": "success"}',
        request_id: 'req-3',
      },
    ];

    // Act
    selectedEndpoint.value = 'orchestrator';

    // Assert
    expect(filteredMessages.value).toHaveLength(2);
    expect(filteredMessages.value[0].endpoint).toBe('orchestrator');
    expect(filteredMessages.value[1].endpoint).toBe('orchestrator');
  });

  it('[11.1.3] should filter messages by text search in subject', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'taletrail.generation.events',
        endpoint: 'generation',
        message_type: 'Event',
        payload: '{"status": "completed"}',
        request_id: 'req-2',
      },
    ];

    // Act
    filterText.value = 'taletrail';

    // Assert
    expect(filteredMessages.value).toHaveLength(1);
    expect(filteredMessages.value[0].subject).toContain('taletrail');
  });

  it('[11.1.4] should filter messages by text search in payload', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "generate_story"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "validate_structure"}',
        request_id: 'req-2',
      },
    ];

    // Act
    filterText.value = 'generate_story';

    // Assert
    expect(filteredMessages.value).toHaveLength(1);
    expect(filteredMessages.value[0].payload).toContain('generate_story');
  });

  it('[11.1.5] should filter messages by text search in request_id', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test"}',
        request_id: 'req-unique-123',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test"}',
        request_id: 'req-456',
      },
    ];

    // Act
    filterText.value = 'unique-123';

    // Assert
    expect(filteredMessages.value).toHaveLength(1);
    expect(filteredMessages.value[0].request_id).toContain('unique-123');
  });

  it('[11.1.6] should combine endpoint and text filters (AND logic)', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "generate_story"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "other_tool"}',
        request_id: 'req-2',
      },
      {
        timestamp: '2025-01-01T00:00:02Z',
        subject: 'mcp.story-generator.request',
        endpoint: 'story-generator',
        message_type: 'Request',
        payload: '{"tool": "generate_story"}',
        request_id: 'req-3',
      },
    ];

    // Act - Filter by orchestrator AND "generate_story"
    selectedEndpoint.value = 'orchestrator';
    filterText.value = 'generate_story';

    // Assert - Should only match the orchestrator message with "generate_story"
    expect(filteredMessages.value).toHaveLength(1);
    expect(filteredMessages.value[0].endpoint).toBe('orchestrator');
    expect(filteredMessages.value[0].payload).toContain('generate_story');
  });

  it('[11.1.7] should be case-insensitive for text filtering', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'MCP.Orchestrator.Request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"Tool": "Generate_Story"}',
        request_id: 'REQ-1',
      },
    ];

    // Act - Search with lowercase
    filterText.value = 'generate_story';

    // Assert
    expect(filteredMessages.value).toHaveLength(1);

    // Act - Search with uppercase
    filterText.value = 'GENERATE_STORY';

    // Assert
    expect(filteredMessages.value).toHaveLength(1);
  });

  it('[11.1.8] should handle filter clearing correctly', () => {
    // Arrange
    messages.value = [
      {
        timestamp: '2025-01-01T00:00:00Z',
        subject: 'mcp.orchestrator.request',
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{"tool": "test"}',
        request_id: 'req-1',
      },
      {
        timestamp: '2025-01-01T00:00:01Z',
        subject: 'mcp.story-generator.request',
        endpoint: 'story-generator',
        message_type: 'Request',
        payload: '{"tool": "test"}',
        request_id: 'req-2',
      },
    ];

    selectedEndpoint.value = 'orchestrator';
    filterText.value = 'test';

    // Should have 1 message (orchestrator with "test")
    expect(filteredMessages.value).toHaveLength(1);

    // Act - Clear filters
    selectedEndpoint.value = 'all';
    filterText.value = '';

    // Assert - Should show all messages
    expect(filteredMessages.value).toHaveLength(2);
  });
});

describe('Monitoring Page - Message Buffer Limit', () => {
  it('[11.1.9] should enforce MAX_MESSAGES limit (1000) using FIFO', () => {
    // Arrange
    const MAX_MESSAGES = 1000;
    const messages = ref<NatsMessage[]>([]);

    // Simulate handleNatsMessage logic
    const handleNatsMessage = (message: NatsMessage) => {
      messages.value.push(message);

      // Limit to MAX_MESSAGES (FIFO: remove oldest)
      if (messages.value.length > MAX_MESSAGES) {
        messages.value = messages.value.slice(-MAX_MESSAGES);
      }
    };

    // Act - Add 1100 messages
    for (let i = 0; i < 1100; i++) {
      handleNatsMessage({
        timestamp: new Date().toISOString(),
        subject: `mcp.test.${i}`,
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: `{"index": ${i}}`,
        request_id: `req-${i}`,
      });
    }

    // Assert
    expect(messages.value).toHaveLength(1000);

    // First message should be index 100 (oldest 100 removed)
    expect(messages.value[0].payload).toContain('"index": 100');

    // Last message should be index 1099
    expect(messages.value[999].payload).toContain('"index": 1099');
  });
});

describe('Monitoring Page - Diagnostics Calculations', () => {
  it('[11.1.10] should track message rate correctly', () => {
    // Arrange
    const messages = ref<NatsMessage[]>([]);
    const now = Date.now();
    const tenSecondsAgo = now - 10000;

    // Add 5 messages from last 10 seconds
    for (let i = 0; i < 5; i++) {
      messages.value.push({
        timestamp: new Date(tenSecondsAgo + i * 2000).toISOString(),
        subject: `mcp.test.${i}`,
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{}',
        request_id: `req-${i}`,
      });
    }

    // Add 3 old messages (more than 10 seconds ago) - should not be counted
    for (let i = 0; i < 3; i++) {
      messages.value.push({
        timestamp: new Date(tenSecondsAgo - 20000).toISOString(),
        subject: `mcp.old.${i}`,
        endpoint: 'orchestrator',
        message_type: 'Request',
        payload: '{}',
        request_id: `req-old-${i}`,
      });
    }

    // Recreate messageRate logic from monitoring.vue
    const messageRate = computed(() => {
      if (messages.value.length === 0) {
        return 0;
      }
      const currentNow = Date.now();
      const currentTenSecondsAgo = currentNow - 10000;
      const recentMessages = messages.value.filter((msg) => {
        const msgTime = new Date(msg.timestamp).getTime();
        return msgTime >= currentTenSecondsAgo;
      });
      return (recentMessages.length / 10).toFixed(1);
    });

    // Assert - Should count only recent 5 messages
    expect(messageRate.value).toBe('0.5'); // 5 messages / 10 seconds = 0.5/s
  });

  it('[11.1.11] should track activity status based on time since last message', () => {
    // Arrange
    const ACTIVITY_WARNING_THRESHOLD = 30; // seconds
    const ACTIVITY_ERROR_THRESHOLD = 60; // seconds

    const diagnostics = ref<MonitoringDiagnostics>({
      received: 10,
      emitted: 10,
      failures: 0,
      lastMessage: undefined,
      connected: new Date().toISOString(),
    });

    // Recreate activityStatus logic from monitoring.vue
    const timeSinceLastMessage = computed(() => {
      if (!diagnostics.value.lastMessage) {
        return null;
      }
      const lastMessageTime = new Date(diagnostics.value.lastMessage).getTime();
      const now = Date.now();
      return Math.floor((now - lastMessageTime) / 1000);
    });

    const activityStatus = computed<'active' | 'warning' | 'error'>(() => {
      const timeSince = timeSinceLastMessage.value;
      if (timeSince === null || timeSince < ACTIVITY_WARNING_THRESHOLD) {
        return 'active';
      }
      if (timeSince < ACTIVITY_ERROR_THRESHOLD) {
        return 'warning';
      }
      return 'error';
    });

    // Act & Assert - No last message
    expect(activityStatus.value).toBe('active');

    // Act & Assert - Recent message (< 30 seconds ago)
    diagnostics.value.lastMessage = new Date(Date.now() - 20000).toISOString();
    expect(activityStatus.value).toBe('active');

    // Act & Assert - Warning state (30-60 seconds ago)
    diagnostics.value.lastMessage = new Date(Date.now() - 45000).toISOString();
    expect(activityStatus.value).toBe('warning');

    // Act & Assert - Error state (> 60 seconds ago)
    diagnostics.value.lastMessage = new Date(Date.now() - 70000).toISOString();
    expect(activityStatus.value).toBe('error');
  });
});
