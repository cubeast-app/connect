use btleplug::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BluetoothError {}

impl BluetoothError {}

impl From<btleplug::Error> for BluetoothError {
    fn from(value: btleplug::Error) -> Self {
        match value {
            Error::PermissionDenied => todo!(),
            Error::DeviceNotFound => todo!(),
            Error::NotConnected => todo!(),
            Error::NotSupported(_) => todo!(),
            Error::TimedOut(_) => todo!(),
            Error::Uuid(_) => todo!(),
            Error::InvalidBDAddr(_) => todo!(),
            Error::Other(_) => todo!(),
        }
    }
}
