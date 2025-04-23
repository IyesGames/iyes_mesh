use std::io::BufWriter;

use iyes_mesh::{descriptor::*, mesh::MeshDataRef, write::*};
use rapidhash::RapidHashMap;

static POSITIONS: &[f32] = &[
    // Front face
    -1.0, -1.0,  1.0,   1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,  -1.0,  1.0,  1.0,
    // Back face
    -1.0, -1.0, -1.0,   1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,  -1.0,  1.0, -1.0,
];

static NORMALS: &[f32] = &[
    // Front face
     0.0,  0.0,  1.0,   0.0,  0.0,  1.0,
     0.0,  0.0,  1.0,   0.0,  0.0,  1.0,
    // Back face
     0.0,  0.0, -1.0,   0.0,  0.0, -1.0,
     0.0,  0.0, -1.0,   0.0,  0.0, -1.0,
];

static UVS: &[f32] = &[
    // Front face
     0.0,  0.0,   0.0,  1.0,
     1.0,  0.0,   1.0,  1.0,
    // Back face
     1.0,  1.0,   1.0,  0.0,
     0.0,  1.0,   0.0,  0.0,
];

static COLORS: &[f32] = &[
    // Front face
     0.0,  0.0,  0.0,  1.0,
     1.0,  0.0,  0.0,  1.0,
     0.0,  1.0,  0.0,  1.0,
     0.0,  0.0,  1.0,  1.0,
    // Back face
     1.0,  1.0,  1.0,  1.0,
     0.0,  1.0,  1.0,  1.0,
     1.0,  0.0,  1.0,  1.0,
     1.0,  1.0,  0.0,  1.0,
];

static INDICES: &[u16] = &[
    // Front face
    0, 1, 2, 2, 3, 0,
    // Back face
    4, 5, 6, 6, 7, 4,
    // Left face
    4, 0, 3, 3, 7, 4,
    // Right face
    1, 5, 6, 6, 2, 1,
    // Top face
    3, 2, 6, 6, 7, 3,
    // Bottom face
    4, 5, 1, 1, 0, 4,
];

fn main() -> anyhow::Result<()> {
    let userdata = b"Hello World!";
    let mut attributes = RapidHashMap::default();
    attributes.insert(
        VertexUsage::Position,
        (VertexFormat::Float32x3, bytemuck::cast_slice(POSITIONS))
    );
    attributes.insert(
        VertexUsage::Normal,
        (VertexFormat::Float32x3, bytemuck::cast_slice(NORMALS))
    );
    attributes.insert(
        VertexUsage::Uv0,
        (VertexFormat::Float32x2, bytemuck::cast_slice(UVS))
    );
    attributes.insert(
        VertexUsage::Color,
        (VertexFormat::Float32x4, bytemuck::cast_slice(COLORS))
    );
    let meshref = MeshDataRef {
        indices: Some((IndexFormat::U16, bytemuck::cast_slice(INDICES))),
        attributes,
    };
    let file = std::fs::File::create("test.ima")?;
    let mut bufw = BufWriter::new(file);
    IyesMeshWriter::new()
        .with_mesh(meshref)?
        .with_user_data(userdata)
        .write_to(&mut bufw)?;
    Ok(())
}
