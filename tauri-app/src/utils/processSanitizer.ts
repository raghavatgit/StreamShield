/**
 * Sanitizes and truncates Windows process names and window titles.
 */
export function sanitizeWindowTitle(title: string, maxLength: number = 48): string {
  if (!title) return 'Untitled Window';

  // Strip null terminators or trailing whitespace
  const cleaned = title.replace(/[\0]/g, '').trim();
  if (cleaned.length <= maxLength) {
    return cleaned;
  }

  return `${cleaned.slice(0, maxLength - 3)}...`;
}

/**
 * Formats process image names into readable titles.
 */
export function formatProcessName(binaryName: string): string {
  if (!binaryName) return 'Unknown Process';

  const base = binaryName.toLowerCase().replace(/\.exe$/, '');
  return base.charAt(0).toUpperCase() + base.slice(1);
}
