/**
 * Window capture affinity mode options.
 */
export type AffinityMode = 'None' | 'Monitor' | 'ExcludeFromCapture';

/**
 * Metadata representation of a running desktop application window.
 */
export interface ShieldedWindow {
  /** Native Windows HWND handle represented as a numerical pointer */
  hwnd: number;
  /** Process identifier (PID) */
  pid: number;
  /** Title text from window caption bar */
  title: string;
  /** Executable binary name (e.g. 'discord.exe') */
  processName: string;
  /** Whether the window is currently shielded from capture buffers */
  isShielded: boolean;
  /** Preferred capture masking mode */
  affinityMode: AffinityMode;
}

/**
 * Telemetry status emitted by the background watchdog daemon.
 */
export interface WatchdogTelemetry {
  activeShieldedCount: number;
  scanIntervalMs: number;
  lastScanTimestamp: number;
}
