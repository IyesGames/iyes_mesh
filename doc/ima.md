# Iyes Mesh Array

## Overview

IMA is a format for storing GPU mesh data. Specifically, an "array" of one
or more compatible (same set of buffers, same vertex/index formats) meshes
concatenated together.

This makes it very convenient for "multi-draw-indirect" use cases for efficient
rendering. The file contains all the values you need to construct an indirect
draw buffer for the GPU (start vertex/index, vertex/index count, base vertex).

Of course, the file format can also be used to simply store just one mesh, as
with other 3D mesh formats.

The data is compressed using zstd.

## General Structure

- Header
- Descriptor
- Data

## Header

 - `[u8; 4]`: Magic: ASCII "IyMA"
 - u16 LE: version = 1
 - u16 LE: descriptor len
 - u64 LE: metadata checksum
 - u64 LE: data checksum

The metadata checksum is computed from:
 - Descriptor encoded bytes
 - Descriptor length
 - Data checksum

(concatenated in this order)

## Descriptor

The following Rust data is to be encoded using `bitcode`:

```rust
struct IyesMeshDescriptor {
    n_vertices: u32,
    user_data_len: u32,
    meshes: Vec<MeshInfo>,
    indices: Option<IndicesInfo>,
    attributes: Vec<VertexAttributeInfo>,
}

struct MeshInfo {
    first_index: u32,
    index_count: u32,
    first_vertex: u32,
    vertex_count: u32,
}

struct IndicesInfo {
    n_indices: u32,
    format: IndexFormat,
}

struct VertexAttributeInfo {
    usage: VertexUsage,
    format: VertexFormat,
}

enum VertexUsage {
    Custom(u32, String),
    Position,
    Normal,
    Tangent,
    Color,
    Uv,
    JointIndex,
    JointWeight,
}

enum IndexFormat {
    U16,
    U32,
}

enum VertexFormat {
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
```

## Data

The data is stored as a single large zstd-compressed stream.

The stream contains all the data buffers concatenated in this order:
 - User Data
 - Index Buffer (if any)
 - Vertex Buffers (in the order listed in the descriptor)

The user data being at the start makes it possible to load only it,
without any of the mesh data.

The mesh buffers are the exact data to be loaded into GPU memory.

The user data can be anything. The format allows users to embed arbitrary
custom data into the file. Typically, this would be the properties of the
material the mesh should be rendered with. Application-specific.

The compressed length can be computed as:
 - `file_size - header_length - descriptor_length`

The uncompressed length can be computed as:
 - For the index buffer, if any, compute the expected raw length:
   - `index_format.size() * n_indices`
 - For each vertex buffer, compute the expected raw length:
   - `vertex_format.size() * n_vertices`
 - Sum everything together

Non-standard zstd settings are used:
 - `include_checksum = false`
 - `include_contentsize = false`
 - `include_dictid = false`
 - `include_magicbytes = false`
 - `long_distance_matching = true`
 - `target_cblock_size = None`
 - `pledged_src_size = Some(computed_data_len)`

### Checksums

Checksums are implemented using the RapidHash algorithm with default seed.

## Recommended Vertex Formats

Standard (as used in Bevy):
 - Positions: Float32x3
 - Normals: Float32x3
 - Colors: Float32x4
 - UVs: Float32x2
 - Tangents: Float32x4
 - Joint weight: Float32x4
 - Joint index: Uint16x4

Compact:
 - Positions: Float16x3
 - Normals: Float16x2
 - Colors: Unorm8x4 / Unorm8x3
 - UVs: Unorm16x2 / Unorm8x2
 - Tangents: Float16x2 (or omit and compute at runtime with mikktspace)
 - Joint weight: Float16x4
 - Joint index: Uint16x4
