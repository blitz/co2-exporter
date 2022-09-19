use hidapi::HidDevice;

pub struct DataReader {
    device: HidDevice,
}

#[derive(Debug)]
pub enum DataReaderError {
    OpenError(hidapi::HidError),
    ReadError(hidapi::HidError),
    InvalidPacketLength(usize),
}

impl std::fmt::Display for DataReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataReaderError::OpenError(_) => write!(f, "Failed to open the CO2 monitor"),
            DataReaderError::ReadError(_) => write!(f, "Failed to read data from the CO2 monitor"),
            DataReaderError::InvalidPacketLength(s) => write!(f, "Unexpected packet length {}", s),
        }
    }
}

impl std::error::Error for DataReaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataReaderError::OpenError(e) | DataReaderError::ReadError(e) => Some(e),
            &DataReaderError::InvalidPacketLength(_) => None,
        }
    }
}

impl DataReader {
    pub fn new() -> Result<Self, DataReaderError> {
        let api = hidapi::HidApi::new().map_err(DataReaderError::OpenError)?;

        let (vid, pid) = (0x04d9, 0xa052);
        let device = api.open(vid, pid).map_err(DataReaderError::OpenError)?;

        device
            .send_feature_report(&[0; 8])
            .map_err(DataReaderError::OpenError)?;

        Ok(Self { device })
    }

    pub fn read(&self) -> Result<[u8; 8], DataReaderError> {
        let mut buf = [0u8; 8];

        let data_len = self
            .device
            .read(&mut buf[..])
            .map_err(DataReaderError::ReadError)?;

        if data_len == buf.len() {
            Ok(buf)
        } else {
            Err(DataReaderError::InvalidPacketLength(data_len))
        }
    }
}
