/**
 * Tenant color utilities for consistent badge and UI coloring
 * Hashes tenant IDs to consistent colors for visual identification
 */

export const TENANT_COLORS = [
  'blue',
  'green',
  'purple',
  'orange',
  'pink',
  'cyan',
  'amber',
  'emerald',
  'violet',
  'rose',
  'sky',
  'teal'
] as const

export type TenantColor = typeof TENANT_COLORS[number]

/**
 * Generate a consistent color for a tenant ID using simple hash
 * Same tenant ID always returns the same color
 */
export function getTenantColor(tenantId: string | undefined): TenantColor {
  if (!tenantId || tenantId === 'no-tenant') {
    return 'gray' as TenantColor
  }

  // Simple hash function
  const hash = tenantId.split('').reduce((acc, char) => {
    return acc + char.charCodeAt(0)
  }, 0)

  return TENANT_COLORS[hash % TENANT_COLORS.length]
}

/**
 * Get a display-friendly short version of tenant ID
 */
export function getShortTenantId(tenantId: string | undefined): string {
  if (!tenantId) {
    return 'No Tenant'
  }

  // If tenant ID is a UUID or long string, show first 8 characters
  if (tenantId.length > 12) {
    return tenantId.substring(0, 8) + '...'
  }

  return tenantId
}

/**
 * Get full display name for tenant
 */
export function getTenantDisplayName(tenantId: string | undefined): string {
  if (!tenantId) {
    return 'No Tenant'
  }

  // Check if it's a common pattern like "tenant-123"
  if (tenantId.startsWith('tenant-')) {
    const id = tenantId.substring(7)
    return `Tenant ${id}`
  }

  return tenantId
}
