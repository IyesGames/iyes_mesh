use std::io::{BufReader, Read, Seek, Write};

use zstd::{Decoder, Encoder};

pub trait ReadSeek: Read + Seek {
}

impl<T: Read + Seek> ReadSeek for T {}

pub trait WriteSeek: Write + Seek {
}

impl<T: Write + Seek> WriteSeek for T {}

pub fn new_zstd_encoder<W: Write>(
    writer: W,
    level: i32,
    pledged_size: u64,
) -> std::io::Result<Encoder<'static, W>> {
    let mut encoder = Encoder::new(writer, level)?;
    encoder.include_checksum(false)?;
    encoder.include_contentsize(false)?;
    encoder.include_dictid(false)?;
    encoder.include_magicbytes(false)?;
    encoder.long_distance_matching(true)?;
    encoder.set_target_cblock_size(None)?;
    encoder.set_pledged_src_size(Some(pledged_size))?;
    Ok(encoder)
}

pub fn new_zstd_decoder<R: Read>(
    reader: R,
) -> std::io::Result<Decoder<'static, BufReader<R>>> {
    let mut decoder = Decoder::new(reader)?;
    decoder.include_magicbytes(false)?;
    Ok(decoder)
}
