use std::io::{Read, SeekFrom};

use crate::checksum::checksum_data;
use crate::HashMap;
use crate::descriptor::*;
use crate::header::{IyesMeshHeader, IyesMeshHeaderParseError};
use crate::io::*;
use crate::mesh::MeshDataRef;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("Did not find magic bytes at start of file")]
    BadMagic,
    #[error("Incompatible version of the file format: {0}")]
    BadVersion(u16),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Checksum mismatch")]
    InvalidChecksums,
    #[error("Cannot decode header: {0}")]
    Header(#[from] IyesMeshHeaderParseError),
    #[error("Cannot decode descriptor: {0}")]
    Descriptor(#[from] IyesMeshDescriptorParseError),
    #[error("Data ends too early")]
    NotEnoughData,
    #[error("Unexpected extra data")]
    TooMuchData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IyesMeshReaderSettings {
    pub verify_metadata_checksum: bool,
    pub verify_data_checksum: bool,
}

impl Default for IyesMeshReaderSettings {
    fn default() -> Self {
        Self {
            verify_metadata_checksum: true,
            verify_data_checksum: true,
        }
    }
}

#[derive(Default, Clone)]
pub struct DecodedBuffers<'s> {
    pub user_data: Option<&'s [u8]>,
    pub buf_index: Option<(IndexFormat, &'s [u8])>,
    pub buf_attrs: HashMap<VertexUsage, (VertexFormat, &'s [u8])>,
}

#[derive(Default, Clone)]
pub struct DecodedMeshes<'s> {
    pub meshes: Vec<MeshDataRef<'s>>,
}

pub struct IyesMeshReader<'s> {
    read: Option<&'s mut dyn ReadSeek>,
    header: IyesMeshHeader,
    descriptor: IyesMeshDescriptor,
    buf: Vec<u8>,
    settings: IyesMeshReaderSettings,
}

pub struct IyesMeshReaderWithData {
    descriptor: IyesMeshDescriptor,
    buf: Vec<u8>,
}

impl<'s> IyesMeshReader<'s> {
    pub fn init(read: &'s mut dyn ReadSeek) -> Result<Self, ReadError> {
        Self::init_with_settings(Default::default(), read)
    }

    pub fn init_with_settings(
        settings: IyesMeshReaderSettings,
        read: &'s mut dyn ReadSeek,
    ) -> Result<Self, ReadError> {
        let mut buf = vec![];
        buf.resize(IyesMeshHeader::encoded_len(), 0);
        read.read_exact(&mut buf)?;
        let header = IyesMeshHeader::from_bytes(&buf)?;
        if header.magic != crate::MAGIC {
            return Err(ReadError::BadMagic);
        }
        if header.version != crate::FORMAT_VERSION {
            return Err(ReadError::BadVersion(header.version));
        }
        buf.resize(header.descriptor_len as usize, 0);
        read.read_exact(&mut buf)?;
        if settings.verify_metadata_checksum {
            let actual_metadata_checksum =
                crate::checksum::checksum_metadata(header, &buf);
            if header.metadata_checksum != actual_metadata_checksum {
                return Err(ReadError::InvalidChecksums);
            }
        }
        let descriptor = IyesMeshDescriptor::from_bytes(&buf)?;
        Ok(Self {
            header,
            descriptor,
            read: Some(read),
            buf,
            settings,
        })
    }

    pub fn header(&self) -> &IyesMeshHeader {
        &self.header
    }

    pub fn descriptor(&self) -> &IyesMeshDescriptor {
        &self.descriptor
    }

    pub fn verify_data_checksum(mut self) -> Result<(), ReadError> {
        if self.header.data_checksum == 0 {
            return Ok(());
        }
        let read = self.read.take().unwrap();
        self.buf.clear();
        read.read_to_end(&mut self.buf)?;
        let actual_data_checksum = checksum_data(&self.buf);
        if self.header.data_checksum != actual_data_checksum {
            return Err(ReadError::InvalidChecksums);
        }
        Ok(())
    }

    pub fn read_all_data(
        mut self
    ) -> Result<IyesMeshReaderWithData, ReadError> {
        let read = self.read.take().unwrap();
        if self.settings.verify_data_checksum && self.header.data_checksum != 0
        {
            self.buf.clear();
            read.read_to_end(&mut self.buf)?;
            let actual_data_checksum = checksum_data(&self.buf);
            if self.header.data_checksum != actual_data_checksum {
                return Err(ReadError::InvalidChecksums);
            }
            read.seek(SeekFrom::Start(
                IyesMeshHeader::encoded_len() as u64
                    + self.header.descriptor_len as u64,
            ))?;
        }
        let mut decoder = new_zstd_decoder(read)?;
        self.buf.clear();
        decoder.read_to_end(&mut self.buf)?;
        Ok(IyesMeshReaderWithData {
            descriptor: self.descriptor,
            buf: self.buf,
        })
    }

    pub fn read_user_data(mut self) -> Result<Vec<u8>, ReadError> {
        let read = self.read.take().unwrap();
        if self.settings.verify_data_checksum && self.header.data_checksum != 0
        {
            self.buf.clear();
            read.read_to_end(&mut self.buf)?;
            let actual_data_checksum = checksum_data(&self.buf);
            if self.header.data_checksum != actual_data_checksum {
                return Err(ReadError::InvalidChecksums);
            }
            read.seek(SeekFrom::Start(
                IyesMeshHeader::encoded_len() as u64
                    + self.header.descriptor_len as u64,
            ))?;
        }
        let mut decoder = new_zstd_decoder(read)?;
        self.buf.resize(self.descriptor.user_data_len as usize, 0);
        decoder.read_exact(&mut self.buf)?;
        Ok(self.buf)
    }
}

impl IyesMeshReaderWithData {
    pub fn descriptor(&self) -> &IyesMeshDescriptor {
        &self.descriptor
    }

    pub fn into_flat_buffers(&self) -> Result<DecodedBuffers<'_>, ReadError> {
        let mut out = DecodedBuffers::default();
        let mut data_remain = &self.buf[..];
        if self.descriptor.user_data_len > 0 {
            let size = self.descriptor.user_data_len as usize;
            if data_remain.len() < size {
                return Err(ReadError::NotEnoughData);
            }
            out.user_data = Some(&data_remain[..size]);
            data_remain = &data_remain[size..];
        }
        if let Some(size) = self.descriptor.compute_index_buf_size() {
            let size = size as usize;
            if data_remain.len() < size {
                return Err(ReadError::NotEnoughData);
            }
            out.buf_index = Some((
                self.descriptor.indices.map(|i| i.format).unwrap(),
                &data_remain[..size],
            ));
            data_remain = &data_remain[size..];
        }
        for (usage, format) in self.descriptor.attributes.iter() {
            let size = format.size() * self.descriptor.n_vertices as usize;
            if data_remain.len() < size {
                return Err(ReadError::NotEnoughData);
            }
            out.buf_attrs.insert(*usage, (*format, &data_remain[..size]));
            data_remain = &data_remain[size..];
        }
        if !data_remain.is_empty() {
            return Err(ReadError::TooMuchData);
        }
        Ok(out)
    }

    pub fn into_split_meshes<'a>(
        &self,
        buffers: &DecodedBuffers<'a>,
    ) -> Result<DecodedMeshes<'a>, ReadError> {
        let mut r = DecodedMeshes::default();
        for m in self.descriptor.meshes.iter() {
            let mut mesh = MeshDataRef::default();
            if let Some((ifmt, idata)) = buffers.buf_index {
                let index_offset = m.first_index as usize * ifmt.size();
                let index_len = m.index_count as usize * ifmt.size();
                if idata.len() < index_offset + index_len {
                    return Err(ReadError::NotEnoughData);
                }
                let mesh_idata =
                    &idata[index_offset..(index_offset + index_len)];
                mesh.indices = Some((ifmt, mesh_idata));
                for (vusage, (vfmt, vdata)) in buffers.buf_attrs.iter() {
                    let vertex_offset = m.first_vertex as usize * ifmt.size();
                    let vertex_len = m.vertex_count as usize * ifmt.size();
                    if vdata.len() < vertex_offset + vertex_len {
                        return Err(ReadError::NotEnoughData);
                    }
                    mesh.attributes.insert(
                        *vusage,
                        (*vfmt, &vdata[vertex_offset..(vertex_offset + vertex_len)]),
                    );
                }
            } else {
                for (vusage, (vfmt, vdata)) in buffers.buf_attrs.iter() {
                    let vertex_offset = m.first_vertex as usize * vfmt.size();
                    let vertex_len = m.vertex_count as usize * vfmt.size();
                    if vdata.len() < vertex_offset + vertex_len {
                        return Err(ReadError::NotEnoughData);
                    }
                    mesh.attributes.insert(
                        *vusage,
                        (
                            *vfmt,
                            &vdata[vertex_offset..(vertex_offset + vertex_len)],
                        ),
                    );
                }
            }
            r.meshes.push(mesh);
        }
        Ok(r)
    }
}

pub fn is_iyes_mesh_file(read: &mut dyn ReadSeek) -> Result<bool, ReadError> {
    read.rewind()?;
    let mut magic = [0; 4];
    read.read_exact(&mut magic)?;
    read.rewind()?;
    Ok(magic == crate::MAGIC)
}
