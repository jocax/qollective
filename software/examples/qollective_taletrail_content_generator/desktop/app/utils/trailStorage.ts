/**
 * Trail storage utility for managing recent trails and file path resolution
 */
import type { TrailListItem } from '~/types/trails'

const RECENT_TRAILS_KEY = 'taletrail-recent-trails'
const MAX_RECENT_TRAILS = 10

/**
 * Save a trail to the recent trails list
 * Maintains a maximum of 10 recent trails, newest first
 */
export function saveRecentTrail(trail: TrailListItem): void {
  console.log('[trailStorage] Saving recent trail:', {
    id: trail.id,
    title: trail.title,
    file_path: trail.file_path
  })

  const recent = getRecentTrails()
  const updated = [
    trail,
    ...recent.filter(t => t.id !== trail.id)
  ].slice(0, MAX_RECENT_TRAILS)

  localStorage.setItem(RECENT_TRAILS_KEY, JSON.stringify(updated))

  console.log('[trailStorage] Recent trails count after save:', updated.length)
}

/**
 * Get all recent trails from localStorage
 */
export function getRecentTrails(): TrailListItem[] {
  try {
    const stored = localStorage.getItem(RECENT_TRAILS_KEY)
    return stored ? JSON.parse(stored) : []
  } catch (error) {
    console.error('Failed to load recent trails:', error)
    return []
  }
}

/**
 * Get file path for a specific trail ID
 * Returns null if trail not found in recent trails
 */
export function getTrailFilePath(trailId: string): string | null {
  console.log('[trailStorage] Looking up file path for trail ID:', trailId)

  const recent = getRecentTrails()
  console.log('[trailStorage] Recent trails in storage:', recent.map(t => ({ id: t.id, file_path: t.file_path })))

  const trail = recent.find(t => t.id === trailId)
  const filePath = trail?.file_path || null

  console.log('[trailStorage] Found file path:', filePath)

  return filePath
}

/**
 * Clear all recent trails from localStorage
 */
export function clearRecentTrails(): void {
  console.log('[trailStorage] Clearing all recent trails from localStorage')
  localStorage.removeItem(RECENT_TRAILS_KEY)
  console.log('[trailStorage] Recent trails cleared')
}

/**
 * Remove a specific trail from recent trails
 */
export function removeRecentTrail(trailId: string): void {
  const recent = getRecentTrails()
  const updated = recent.filter(t => t.id !== trailId)
  localStorage.setItem(RECENT_TRAILS_KEY, JSON.stringify(updated))
}
