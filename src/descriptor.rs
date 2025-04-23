use crate::HashMap;

#[derive(Debug, Clone, bitcode::Encode, bitcode::Decode)]
pub struct IyesMeshDescriptor {
    pub n_vertices: u32,
    pub user_data_len: u32,
    pub meshes: Vec<MeshInfo>,
    pub indices: Option<IndicesInfo>,
    pub attributes: HashMap<VertexUsage, VertexFormat>,
}

#[derive(Default, Debug, Clone, Copy, bitcode::Encode, bitcode::Decode)]
pub struct MeshInfo {
    /// First index (if indices present) or vertex (if no indices present)
    pub first: u32,
    /// Number of indices (if present) or vertices (if no indices present)
    pub count: u32,
    /// If indices present, offset to add when indexing
    pub base_vertex: i32,
}

#[derive(Debug, Clone, Copy, bitcode::Encode, bitcode::Decode)]
pub struct IndicesInfo {
    pub n_indices: u32,
    pub format: IndexFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bitcode::Encode, bitcode::Decode)]
pub enum VertexUsage {
    Custom(u32),
    Position,
    Normal,
    Tangent,
    Uv0,
    Uv1,
    JointIndex,
    JointWeight,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bitcode::Encode, bitcode::Decode)]
pub enum IndexFormat {
    U16,
    U32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bitcode::Encode, bitcode::Decode)]
pub enum VertexFormat {
    Float16,
    Float32,
    Float64,
    Float16x2,
    Float16x4,
    Float32x2,
    Float32x3,
    Float32x4,
    Float64x2,
    Float64x3,
    Float64x4,
    Sint8,
    Sint8x2,
    Sint8x4,
    Sint16,
    Sint32,
    Sint16x2,
    Sint16x4,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Snorm8,
    Snorm8x2,
    Snorm8x4,
    Snorm16,
    Snorm16x2,
    Snorm16x4,
    Uint8,
    Uint8x2,
    Uint8x4,
    Uint16,
    Uint32,
    Uint16x2,
    Uint16x4,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Unorm8,
    Unorm8x2,
    Unorm8x4,
    Unorm8x4Bgra,
    Unorm16,
    Unorm10_10_10_2,
    Unorm16x2,
    Unorm16x4,
}

impl IndexFormat {
    /// Returns the byte size of the format.
    pub const fn size(self) -> usize {
        match self {
            IndexFormat::U16 => 2,
            IndexFormat::U32 => 4,
        }
    }
}

impl VertexFormat {
    /// Returns the byte size of the format.
    pub const fn size(self) -> usize {
        match self {
            Self::Uint8 | Self::Sint8 | Self::Unorm8 | Self::Snorm8 => 1,
            Self::Uint8x2
            | Self::Sint8x2
            | Self::Unorm8x2
            | Self::Snorm8x2
            | Self::Uint16
            | Self::Sint16
            | Self::Unorm16
            | Self::Snorm16
            | Self::Float16 => 2,
            Self::Uint8x4
            | Self::Sint8x4
            | Self::Unorm8x4
            | Self::Snorm8x4
            | Self::Uint16x2
            | Self::Sint16x2
            | Self::Unorm16x2
            | Self::Snorm16x2
            | Self::Float16x2
            | Self::Float32
            | Self::Uint32
            | Self::Sint32
            | Self::Unorm10_10_10_2
            | Self::Unorm8x4Bgra => 4,
            Self::Uint16x4
            | Self::Sint16x4
            | Self::Unorm16x4
            | Self::Snorm16x4
            | Self::Float16x4
            | Self::Float32x2
            | Self::Uint32x2
            | Self::Sint32x2
            | Self::Float64 => 8,
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 12,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 | Self::Float64x2 => 16,
            Self::Float64x3 => 24,
            Self::Float64x4 => 32,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IyesMeshDescriptorParseError {
    #[error("Bitcode decode error: {0}")]
    Bitcode(#[from] bitcode::Error),
}

impl IyesMeshDescriptor {
    pub const fn encoded_len() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, IyesMeshDescriptorParseError> {
        let descriptor = bitcode::decode(&buf)?;
        Ok(descriptor)
    }

    pub fn compute_vertex_buf_size(&self, buf: VertexUsage) -> Option<u32> {
        self.attributes.get(&buf).map(|fmt| fmt.size() as u32 * self.n_vertices as u32)
    }

    pub fn compute_index_buf_size(&self) -> Option<u32> {
        self.indices.map(|info| info.format.size() as u32 * info.n_indices as u32)
    }

    pub fn compute_all_vertex_buf_sizes(&self) -> u64 {
        self.attributes.values().map(|fmt| fmt.size() as u64 * self.n_vertices as u64).sum()
    }

    pub fn compute_all_buf_sizes(&self) -> u64 {
        self.compute_index_buf_size().unwrap_or(0) as u64
            + self.compute_all_vertex_buf_sizes()
    }

    pub fn compute_total_raw_data_size(&self) -> u64 {
        self.compute_all_buf_sizes()
            + self.user_data_len as u64
    }
}
