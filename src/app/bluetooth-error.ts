/**
 * Mirrors the Rust `ErrorCategory` enum in src-tauri/src/bluetooth/error.rs.
 *
 * - `system`       — OS/hardware is blocking Bluetooth (grant permissions, enable adapter)
 * - `connectivity` — Bluetooth works but the target device was unreachable (retry, move closer)
 * - `device`       — Device is reachable but doesn't expose the expected services (wrong model / firmware)
 * - `internal`     — Application bug or invalid protocol usage; should not surface in normal use
 */
export type ErrorCategory = 'system' | 'connectivity' | 'device' | 'internal';

export type SystemErrorCode =
  | 'permission_denied'  // OS refused the Bluetooth permission
  | 'no_adapter'         // No Bluetooth adapter detected on this system
  | 'not_supported'      // Operation not supported on this platform
  | 'runtime_error';     // Low-level platform runtime error (incl. OS BT-stack errors)

export type ConnectivityErrorCode =
  | 'device_not_found'   // Peripheral is no longer visible to the adapter
  | 'timed_out'          // BT operation exceeded its time limit
  | 'not_connected';     // Operation requires connection but device is not connected

export type DeviceErrorCode =
  | 'characteristic_not_found'   // UUID not present in the peripheral's services
  | 'unexpected_characteristic'; // Notification received from an unexpected characteristic

export type InternalErrorCode =
  | 'unexpected_callback'  // Platform callback for an unknown operation
  | 'invalid_uuid'         // UUID string could not be parsed
  | 'invalid_address'      // Bluetooth device address is invalid
  | 'invalid_state';       // Client called an operation requiring a different app state

export type ErrorCode =
  | SystemErrorCode
  | ConnectivityErrorCode
  | DeviceErrorCode
  | InternalErrorCode;

/**
 * Structured error payload returned by the WebSocket server when a request fails.
 *
 * Wire format (always accompanies `"result": "error"`):
 * ```json
 * { "result": "error", "category": "connectivity", "code": "device_not_found" }
 * ```
 */
export interface BluetoothError {
  category: ErrorCategory;
  code: ErrorCode;
}
