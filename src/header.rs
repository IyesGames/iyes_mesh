#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct IyesMeshHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub descriptor_len: u16,
    pub metadata_checksum: u64,
    pub data_checksum: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum IyesMeshHeaderParseError {
    #[error("Bytes array cannot be reinterpreted/cast: {0}")]
    Bytemuck(bytemuck::PodCastError),
}

impl IyesMeshHeader {
    pub const fn encoded_len() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, IyesMeshHeaderParseError> {
        let raw_header: &IyesMeshHeader = bytemuck::try_from_bytes(buf)
            .map_err(IyesMeshHeaderParseError::Bytemuck)?;
        Ok(raw_header.to_le())
    }

    pub fn to_le(&self) -> Self {
        Self {
            magic: self.magic,
            version: self.version.to_le(),
            descriptor_len: self.descriptor_len.to_le(),
            metadata_checksum: self.metadata_checksum.to_le(),
            data_checksum: self.data_checksum.to_le(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
