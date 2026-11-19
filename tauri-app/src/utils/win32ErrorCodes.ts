/**
 * Translates native Windows Win32 error codes into human-readable messages.
 */
export function resolveWin32Error(errorCode: number): string {
  switch (errorCode) {
    case 0:
      return 'Success (ERROR_SUCCESS)';
    case 5:
      return 'Access is denied (ERROR_ACCESS_DENIED). Process requires elevated Administrator privileges.';
    case 6:
      return 'Invalid handle (ERROR_INVALID_HANDLE). Target window handle may have closed.';
    case 87:
      return 'The parameter is incorrect (ERROR_INVALID_PARAMETER).';
    case 1400:
      return 'Invalid window handle (ERROR_INVALID_WINDOW_HANDLE). Window was destroyed during injection.';
    case 1460:
      return 'This operation returned because the timeout period expired (ERROR_TIMEOUT).';
    default:
      return `Win32 Error Code: ${errorCode}`;
  }
}

/**
 * Normalizes backend errors (including trailing Win32 numeric error codes like "OpenProcess(1234): 5")
 * into clean user-facing messages.
 */
export function formatBackendError(err: unknown): string {
  const raw = String(err || '').replace(/^Error:\s*/i, '').trim();
  if (!raw) {
    return 'An unexpected error occurred.';
  }

  const codeMatch = raw.match(/(?:[:(]\s*|\berror code\s+)(\d+)\)?$/i);
  if (codeMatch) {
    const code = parseInt(codeMatch[1], 10);
    if (!Number.isNaN(code) && [5, 6, 87, 1400, 1460].includes(code)) {
      return resolveWin32Error(code);
    }
  }

  return raw;
}
