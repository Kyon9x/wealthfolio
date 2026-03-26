/**
 * Simple ID generation utilities.
 * Uses crypto.randomUUID() when available, falls back to timestamp-based IDs.
 */

/** Generate a unique ID */
export function generateId(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }

  // Fallback: timestamp + random string
  return `id_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}
