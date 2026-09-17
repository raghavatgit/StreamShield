import { resolveWin32Error } from './win32ErrorCodes';

describe('resolveWin32Error', () => {
  it('resolves standard success code', () => {
    expect(resolveWin32Error(0)).toContain('Success');
  });

  it('resolves access denied error with UAC hint', () => {
    expect(resolveWin32Error(5)).toContain('Access is denied');
    expect(resolveWin32Error(5)).toContain('Administrator');
  });

  it('resolves invalid window handle error', () => {
    expect(resolveWin32Error(1400)).toContain('Invalid window handle');
  });

  it('formats unknown error codes gracefully', () => {
    expect(resolveWin32Error(9999)).toBe('Win32 Error Code: 9999');
  });
});
