use crate::header::IyesMeshHeader;

#[inline(always)]
pub fn checksum_data(data: &[u8]) -> u64 {
    rapidhash::rapidhash_inline(data, rapidhash::RAPID_SEED)
}

#[inline(always)]
pub fn checksum_metadata(
    header: IyesMeshHeader,
    encoded_descriptor: &[u8],
) -> u64 {
    let hasher = rapidhash::RapidInlineHasher::default_const();
    let hasher = hasher.write_const(&encoded_descriptor);
    let hasher = hasher.write_const(&header.descriptor_len.to_le_bytes());
    let hasher = hasher.write_const(&header.data_checksum.to_le_bytes());
    hasher.finish_const()
}
