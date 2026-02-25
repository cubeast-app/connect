use btleplug::Error as BtleError;
use serde::{Deserialize, Serialize};

/// High-level category — tells the recipient *who* is responsible for the problem
/// and what kind of remediation makes sense.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// The OS/hardware is blocking Bluetooth — the user must act at the system level
    /// (e.g. grant permissions, enable the adapter).
    System,
    /// Bluetooth is working but the target device could not be reached — retry or
    /// reduce distance may help.
    Connectivity,
    /// The peripheral is reachable but does not expose the expected services/characteristics
    /// — likely the wrong device model or an unsupported firmware version.
    Device,
    /// Application-level bug or invalid protocol usage — should never surface in normal use.
    Internal,
}

/// Fine-grained code identifying the precise failure within its category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // --- System ---
    /// The OS refused the Bluetooth permission.
    PermissionDenied,
    /// No Bluetooth adapter was detected on this system.
    NoAdapter,
    /// The requested Bluetooth operation is not supported on this platform.
    NotSupported,
    /// A low-level platform runtime error occurred.
    RuntimeError,

    // --- Connectivity ---
    /// The peripheral is no longer visible to the adapter.
    DeviceNotFound,
    /// A Bluetooth operation exceeded its time limit — device may be out of range.
    TimedOut,
    /// The operation requires an active connection but the device is not connected.
    NotConnected,

    // --- Device ---
    /// The characteristic UUID is not present in the peripheral's services.
    CharacteristicNotFound,
    /// Received a notification from an unexpected characteristic.
    UnexpectedCharacteristic,

    // --- Internal ---
    /// Received a platform callback for an unknown operation.
    UnexpectedCallback,
    /// The provided UUID string could not be parsed.
    InvalidUuid,
    /// The provided Bluetooth device address is invalid.
    InvalidAddress,
    /// The client called an operation that requires a different application state.
    InvalidState,
    /// An unclassified error occurred.
    Unknown,
}

/// Structured error type sent over the wire to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub category: ErrorCategory,
    pub code: ErrorCode,
}

impl AppError {
    /// Create an error representing invalid protocol usage (e.g. calling
    /// StopDiscovery when discovery is not running).
    pub fn invalid_state() -> Self {
        AppError {
            category: ErrorCategory::Internal,
            code: ErrorCode::InvalidState,
        }
    }
}

impl From<BtleError> for AppError {
    fn from(error: BtleError) -> Self {
        let (category, code) = match &error {
            BtleError::PermissionDenied => (ErrorCategory::System, ErrorCode::PermissionDenied),
            BtleError::NotSupported(msg) if msg.contains("No Bluetooth adapters") => {
                (ErrorCategory::System, ErrorCode::NoAdapter)
            }
            BtleError::NotSupported(_) => (ErrorCategory::System, ErrorCode::NotSupported),
            BtleError::RuntimeError(_) => (ErrorCategory::System, ErrorCode::RuntimeError),

            BtleError::DeviceNotFound => (ErrorCategory::Connectivity, ErrorCode::DeviceNotFound),
            BtleError::TimedOut(_) => (ErrorCategory::Connectivity, ErrorCode::TimedOut),
            BtleError::NotConnected => (ErrorCategory::Connectivity, ErrorCode::NotConnected),

            BtleError::NoSuchCharacteristic => {
                (ErrorCategory::Device, ErrorCode::CharacteristicNotFound)
            }
            BtleError::UnexpectedCharacteristic => {
                (ErrorCategory::Device, ErrorCode::UnexpectedCharacteristic)
            }

            BtleError::UnexpectedCallback => {
                (ErrorCategory::Internal, ErrorCode::UnexpectedCallback)
            }
            BtleError::Uuid(_) => (ErrorCategory::Internal, ErrorCode::InvalidUuid),
            BtleError::InvalidBDAddr(_) => (ErrorCategory::Internal, ErrorCode::InvalidAddress),
            // `Other` wraps raw OS/platform BT-stack errors (e.g. BlueZ "Service Discovery
            // timed out"). These are always system-level, not internal application bugs.
            BtleError::Other(_) => (ErrorCategory::System, ErrorCode::RuntimeError),
        };

        AppError { category, code }
    }
}
