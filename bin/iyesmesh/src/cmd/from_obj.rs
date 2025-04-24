use std::io::{BufReader, BufWriter};

use iyes_mesh::HashMap;
use iyes_mesh::descriptor::{IndexFormat, VertexFormat, VertexUsage};
use iyes_mesh::mesh::MeshDataRef;
use iyes_mesh::read::{IyesMeshReader, IyesMeshReaderSettings};
use iyes_mesh::write::{IyesMeshWriter, IyesMeshWriterSettings};
use obj::raw::{RawObj, parse_obj};
use obj::{Obj, Position, TexturedVertex, Vertex};

use crate::CommonArgs;
use crate::prelude::*;
use crate::util::load_user_data;

#[derive(clap::Args, Debug)]
pub struct FromObjArgs {
    /// File to load user data from (stdin if unspecified)
    ///
    /// If the file is an IMA file, extract the user data from it.
    /// If the file is not an IMA file, use its raw contents as-is.
    #[arg(short, long)]
    user_data: Option<Option<PathBuf>>,
    /// If a user data file is provided, do not try to parse it as an IMA file
    #[arg(long)]
    user_data_force_raw: bool,
    /// If the output IMA file exists, try to add the new mesh to it
    #[arg(short, long)]
    append: bool,
    #[command(flatten)]
    rarg: crate::ReadArgs,
    #[command(flatten)]
    warg: crate::WriteArgs,
    #[command(flatten)]
    oarg: crate::OutputArgs,
    #[command(flatten)]
    outpath: crate::OutputPath,
    #[command(flatten)]
    inpaths: crate::InputPaths,
}

pub fn run(
    _args_common: &CommonArgs,
    args_cmd: &FromObjArgs,
) -> AnyResult<()> {
    if args_cmd.inpaths.in_files.is_empty() {
        bail!("No input files provided.");
    }
    let mut writer = IyesMeshWriter::new_with_settings(
        IyesMeshWriterSettings::from(&args_cmd.warg),
    );
    let new_user_data;
    match &args_cmd.user_data {
        Some(src) => {
            new_user_data = load_user_data(
                src.as_deref(),
                IyesMeshReaderSettings::from(&args_cmd.rarg),
                args_cmd.user_data_force_raw,
            )?;
            writer.set_user_data(&new_user_data);
        }
        None => {}
    }

    let mut bufs = vec![];
    let mut new_meshes = vec![];

    for path in args_cmd.inpaths.in_files.iter() {
        let mut bi = vec![];
        let mut bp = vec![];
        let mut bn = vec![];
        let mut bt = vec![];
        let infile = std::fs::File::open(&path)
            .context("Cannot open input OBJ file")?;
        let bufr = BufReader::new(infile);
        let rawobj = parse_obj(bufr).context("Cannot parse OBJ file")?;
        let ifmt = try_ptn16(rawobj.clone(), &mut bi, &mut bp, &mut bt, &mut bn)
            .or_else(|_| {
                try_ptn32(rawobj.clone(), &mut bi, &mut bp, &mut bt, &mut bn)
            })
            .or_else(|_| try_pn16(rawobj.clone(), &mut bi, &mut bp, &mut bn))
            .or_else(|_| try_pn32(rawobj.clone(), &mut bi, &mut bp, &mut bn))
            .or_else(|_| try_p16(rawobj.clone(), &mut bi, &mut bp))
            .or_else(|_| try_p32(rawobj.clone(), &mut bi, &mut bp))
            .context("OBJ file is not in any valid vertex format")?;

        bufs.push((ifmt, bi, bp, bn, bt));
    }
    for (ifmt, bi, bp, bn, bt) in bufs.iter() {
        let mut attributes = HashMap::default();
        if !bp.is_empty() {
            attributes.insert(
                VertexUsage::Position,
                (VertexFormat::Float32x3, bp.as_slice()),
            );
        } else {
            bail!("No vertex positions!");
        }
        if !bn.is_empty() {
            attributes.insert(
                VertexUsage::Normal,
                (VertexFormat::Float32x3, bn.as_slice()),
            );
        }
        if !bt.is_empty() {
            attributes
                .insert(VertexUsage::Uv0, (VertexFormat::Float32x2, bt.as_slice()));
        }
        let mesh = MeshDataRef {
            indices: Some((*ifmt, &bi)),
            attributes,
        };

        new_meshes.push(mesh);
    }

    let with_data;
    let flatbufs;
    let meshes;
    if args_cmd.append {
        let mut infile = std::fs::File::open(&args_cmd.outpath.out_file)
            .context("Could not open input file")?;
        let reader = IyesMeshReader::init_with_settings(
            IyesMeshReaderSettings::from(&args_cmd.rarg),
            &mut infile,
        )
        .context("Cannot decode append file metadata and initialize decoding")?;
        with_data =
            reader.read_all_data().context("Cannot decode append file data")?;
        flatbufs =
            with_data.into_flat_buffers().context("Cannot decode append file buffers")?;
        meshes = with_data
            .into_split_meshes(&flatbufs)
            .context("Cannot decode append file meshes")?;
        for m in meshes.meshes.iter() {
            writer.add_mesh(m.clone()).context("Cannot use old mesh for output")?;
        }
    }

    for m in new_meshes {
        writer.add_mesh(m).context("New mesh is incompatible")?;
    }

    let outfile = if args_cmd.oarg.overwrite {
        std::fs::File::create(&args_cmd.outpath.out_file)
            .context("Could not open output file")?
    } else {
        std::fs::File::create_new(&args_cmd.outpath.out_file)
            .context("Could not open output file")?
    };
    let mut bufout = BufWriter::new(outfile);
    writer.write_to(&mut bufout).context("Cannot encode output file")?;

    Ok(())
}

fn try_ptn16(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
    bt: &mut Vec<u8>,
    bn: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<TexturedVertex, u16> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
        bt.extend_from_slice(&v.texture[0].to_le_bytes());
        bt.extend_from_slice(&v.texture[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[0].to_le_bytes());
        bn.extend_from_slice(&v.normal[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[2].to_le_bytes());
    }
    Ok(IndexFormat::U16)
}

fn try_ptn32(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
    bt: &mut Vec<u8>,
    bn: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<TexturedVertex, u32> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
        bt.extend_from_slice(&v.texture[0].to_le_bytes());
        bt.extend_from_slice(&v.texture[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[0].to_le_bytes());
        bn.extend_from_slice(&v.normal[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[2].to_le_bytes());
    }
    Ok(IndexFormat::U32)
}

fn try_pn16(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
    bn: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<Vertex, u16> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
        bn.extend_from_slice(&v.normal[0].to_le_bytes());
        bn.extend_from_slice(&v.normal[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[2].to_le_bytes());
    }
    Ok(IndexFormat::U16)
}

fn try_pn32(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
    bn: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<Vertex, u32> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
        bn.extend_from_slice(&v.normal[0].to_le_bytes());
        bn.extend_from_slice(&v.normal[1].to_le_bytes());
        bn.extend_from_slice(&v.normal[2].to_le_bytes());
    }
    Ok(IndexFormat::U32)
}

fn try_p16(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<Position, u16> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
    }
    Ok(IndexFormat::U16)
}

fn try_p32(
    rawobj: RawObj,
    bi: &mut Vec<u8>,
    bp: &mut Vec<u8>,
) -> AnyResult<IndexFormat> {
    let obj: Obj<Position, u32> = Obj::new(rawobj.clone())?;
    for i in obj.indices {
        bi.extend_from_slice(&i.to_le_bytes());
    }
    for v in obj.vertices {
        bp.extend_from_slice(&v.position[0].to_le_bytes());
        bp.extend_from_slice(&v.position[1].to_le_bytes());
        bp.extend_from_slice(&v.position[2].to_le_bytes());
    }
    Ok(IndexFormat::U32)
}
