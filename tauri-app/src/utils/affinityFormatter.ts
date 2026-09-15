import { AffinityMode } from '../types/shield';

export interface AffinityBadgeInfo {
  label: string;
  badgeClass: string;
  description: string;
}

/**
 * Returns formatted metadata and visual badge classes for a given affinity mode.
 */
export function formatAffinityMode(mode: AffinityMode): AffinityBadgeInfo {
  switch (mode) {
    case 'ExcludeFromCapture':
      return {
        label: 'Fully Masked',
        badgeClass: 'badge-emerald',
        description: 'Excluded from screen capture feeds and recording software.'
      };
    case 'Monitor':
      return {
        label: 'Monitor Only',
        badgeClass: 'badge-amber',
        description: 'Blacked out on capture feeds; visible on primary display.'
      };
    case 'None':
    default:
      return {
        label: 'Unshielded',
        badgeClass: 'badge-slate',
        description: 'Visible on all screen capture feeds and local monitors.'
      };
  }
}
