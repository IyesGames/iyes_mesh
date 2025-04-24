use std::io::Write;

use crate::HashMap;
use crate::descriptor::*;
use crate::header::IyesMeshHeader;
use crate::io::*;
use crate::mesh::*;

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Mesh Data")]
    InvalidMesh,
    #[error("Meshes must have an identical set of buffers and formats")]
    IncompatibleMeshes,
    #[error("No source meshes provided")]
    NoMeshes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IyesMeshWriterSettings {
    /// Convert U16 indices to U32 indices if necessary.
    pub upconvert_indices: bool,
    /// If false, will write zero in place of the data checksum
    ///
    /// Disabling this is a big performance improvement, as there is no need
    /// to go through all the data again after encoding, to compute a
    /// checksum. It also allows the file to be written as a single pass.
    pub write_data_checksum: bool,
    /// Zstd compression level.
    pub compression_level: i32,
}

impl Default for IyesMeshWriterSettings {
    fn default() -> Self {
        Self {
            upconvert_indices: false,
            write_data_checksum: true,
            compression_level: *zstd::compression_level_range().end(),
        }
    }
}

pub struct IyesMeshWriter<'s> {
    user_data: Option<&'s [u8]>,
    settings: IyesMeshWriterSettings,
    src_meshes: Vec<MeshDataRef<'s>>,
    scratch: Vec<u8>,
}

impl<'s> IyesMeshWriter<'s> {
    pub fn new() -> Self {
        Self::new_with_settings(Default::default())
    }

    pub fn new_with_settings(settings: IyesMeshWriterSettings) -> Self {
        Self {
            settings,
            user_data: None,
            src_meshes: vec![],
            scratch: vec![],
        }
    }

    pub fn set_user_data(
        &mut self,
        user_data: &'s [u8],
    ) {
        self.user_data = Some(user_data);
    }

    pub fn clear_user_data(&mut self) {
        self.user_data = None;
    }

    pub fn with_user_data(
        mut self,
        user_data: &'s [u8],
    ) -> Self {
        self.set_user_data(user_data);
        self
    }

    pub fn without_user_data(mut self) -> Self {
        self.clear_user_data();
        self
    }

    pub fn add_mesh(
        &mut self,
        mesh: MeshDataRef<'s>,
    ) -> Result<(), WriteError> {
        if !mesh.validate() {
            return Err(WriteError::InvalidMesh);
        }
        self.src_meshes.push(mesh);
        Ok(())
    }

    pub fn with_mesh(
        mut self,
        mesh: MeshDataRef<'s>,
    ) -> Result<Self, WriteError> {
        self.add_mesh(mesh)?;
        Ok(self)
    }

    fn scan_needed_buffers(&self) -> Result<HaveBuffers, WriteError> {
        let mut iter = self.src_meshes.iter();
        let first = iter.next().ok_or(WriteError::NoMeshes)?;
        let mut r = HaveBuffers {
            indices: first.indices.map(|b| b.0),
            attrs: first.attributes.iter().map(|b| (*b.0, b.1.0)).collect(),
        };
        for m in iter {
            match (m.indices.map(|b| b.0), r.indices) {
                (None, None)
                | (Some(IndexFormat::U16), Some(IndexFormat::U16))
                | (Some(IndexFormat::U32), Some(IndexFormat::U32)) => {}
                (Some(IndexFormat::U16), Some(IndexFormat::U32)) => {
                    if !self.settings.upconvert_indices {
                        return Err(WriteError::IncompatibleMeshes);
                    }
                }
                (Some(IndexFormat::U32), Some(IndexFormat::U16)) => {
                    if !self.settings.upconvert_indices {
                        return Err(WriteError::IncompatibleMeshes);
                    }
                    r.indices = Some(IndexFormat::U32);
                }
                _ => return Err(WriteError::IncompatibleMeshes),
            }
            if !m.attributes.iter().all(|b| r.attrs.get(b.0) == Some(&b.1.0)) {
                return Err(WriteError::IncompatibleMeshes);
            }
            if !r.attrs.iter().all(|br| {
                m.attributes
                    .iter()
                    .find(|bm| bm.0 == br.0 && bm.1.0 == *br.1)
                    .is_some()
            }) {
                return Err(WriteError::IncompatibleMeshes);
            }
        }
        Ok(r)
    }

    fn compute_uncompressed_sizes(
        &self,
        upconverting_indices: bool,
    ) -> u64 {
        let mut total = 0;
        for m in self.src_meshes.iter() {
            if let Some(b) = m.indices {
                if b.0 == IndexFormat::U16 && upconverting_indices {
                    total += b.1.len() as u64 * 2;
                } else {
                    total += b.1.len() as u64;
                }
            }
            for b in m.attributes.iter() {
                total += b.1.1.len() as u64;
            }
        }
        total
    }

    fn gen_meshinfo(
        &self,
        has_indices: bool,
    ) -> Vec<MeshInfo> {
        let mut r = Vec::with_capacity(self.src_meshes.len());
        let mut base_vertex = 0;
        let mut first = 0;
        for m in self.src_meshes.iter() {
            if has_indices {
                let n_indices = m.n_indices().unwrap() as u32;
                let n_vertices = m.n_vertices() as u32;
                r.push(MeshInfo {
                    first_index: first,
                    index_count: n_indices,
                    first_vertex: base_vertex,
                    vertex_count: n_vertices,
                });
                first += n_indices;
                base_vertex += n_vertices;
            } else {
                let n_vertices = m.n_vertices() as u32;
                r.push(MeshInfo {
                    first_index: 0,
                    index_count: 0,
                    first_vertex: first,
                    vertex_count: n_vertices,
                });
                first += n_vertices;
            }
        }
        r
    }

    pub fn write_to(
        mut self,
        write: &'s mut dyn WriteSeek,
    ) -> Result<(), WriteError> {
        let havebufs = self.scan_needed_buffers()?;
        let computed_bufsizes = self.compute_uncompressed_sizes(
            self.settings.upconvert_indices
                && havebufs.indices == Some(IndexFormat::U32),
        );
        let n_vertices: usize =
            self.src_meshes.iter().map(|m| m.n_vertices()).sum();
        let n_indices: usize =
            self.src_meshes.iter().filter_map(|m| m.n_indices()).sum();
        let descriptor = IyesMeshDescriptor {
            n_vertices: n_vertices as u32,
            user_data_len: self.user_data.map(|b| b.len() as u32).unwrap_or(0),
            meshes: self.gen_meshinfo(havebufs.indices.is_some()),
            indices: havebufs.indices.map(|format| IndicesInfo {
                n_indices: n_indices as u32,
                format,
            }),
            attributes: havebufs.attrs.clone(),
        };
        let bytes_descriptor = bitcode::encode(&descriptor);
        let mut header = IyesMeshHeader {
            magic: crate::MAGIC,
            version: crate::FORMAT_VERSION,
            descriptor_len: bytes_descriptor.len() as u16,
            data_checksum: 0,
            metadata_checksum: 0,
        };
        let total_uncompressed_len =
            computed_bufsizes + descriptor.user_data_len as u64;
        if self.settings.write_data_checksum {
            let mut comprbuf = vec![];
            let encoder = new_zstd_encoder(
                &mut comprbuf,
                self.settings.compression_level,
                total_uncompressed_len,
            )?;
            self.do_encode_data(&descriptor, encoder)?;
            header.data_checksum = crate::checksum::checksum_data(&comprbuf);
            header.metadata_checksum =
                crate::checksum::checksum_metadata(header, &bytes_descriptor);
            write.write_all(header.as_bytes())?;
            write.write_all(&bytes_descriptor)?;
            write.write_all(&comprbuf)?;
        } else {
            header.metadata_checksum =
                crate::checksum::checksum_metadata(header, &bytes_descriptor);
            write.write_all(header.as_bytes())?;
            write.write_all(&bytes_descriptor)?;
            let encoder = new_zstd_encoder(
                write,
                self.settings.compression_level,
                total_uncompressed_len,
            )?;
            self.do_encode_data(&descriptor, encoder)?;
        }
        Ok(())
    }

    fn do_encode_data<W: Write>(
        &mut self,
        descriptor: &IyesMeshDescriptor,
        mut encoder: zstd::Encoder<'static, W>,
    ) -> Result<W, WriteError> {
        if let Some(user_data) = self.user_data {
            encoder.write_all(user_data)?;
        }
        if let Some(info) = &descriptor.indices {
            for bb in self.src_meshes.iter() {
                let (fmt, bytes) = bb.indices.unwrap();
                if self.settings.upconvert_indices
                    && fmt == IndexFormat::U16
                    && info.format == IndexFormat::U32
                {
                    self.scratch.clear();
                    self.scratch.reserve(bytes.len() * 2);
                    for rb in bytes.chunks_exact(2) {
                        let nb = (u16::from_le_bytes([rb[0], rb[1]]) as u32)
                            .to_le_bytes();
                        self.scratch.extend_from_slice(&nb);
                    }
                    encoder.write_all(&self.scratch)?;
                } else {
                    encoder.write_all(bytes)?;
                }
            }
        }
        for attr in descriptor.attributes.iter() {
            for bb in self.src_meshes.iter() {
                let (_, bytes) = bb.attributes[attr.0];
                encoder.write_all(bytes)?;
            }
        }
        let write = encoder.finish()?;
        Ok(write)
    }
}

struct HaveBuffers {
    indices: Option<IndexFormat>,
    attrs: HashMap<VertexUsage, VertexFormat>,
}
