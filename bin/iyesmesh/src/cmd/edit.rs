use std::io::BufWriter;

use iyes_mesh::HashSet;
use iyes_mesh::read::{
    IyesMeshReader, IyesMeshReaderSettings,
};
use iyes_mesh::write::{IyesMeshWriter, IyesMeshWriterSettings};

use crate::CommonArgs;
use crate::prelude::*;
use crate::util::load_user_data;

#[derive(clap::Args, Debug)]
pub struct EditArgs {
    /// Replace user data from file (stdin if unspecified)
    ///
    /// If the file is an IMA file, extract the user data from it.
    /// If the file is not an IMA file, use its raw contents as-is.
    #[arg(short, long)]
    user_data: Option<Option<PathBuf>>,
    /// If a user data file is provided, do not try to parse it as an IMA file
    #[arg(long)]
    user_data_force_raw: bool,
    /// Delete existing user data
    #[arg(short = 'D', long)]
    drop_user_data: bool,
    /// Delete specific meshes
    #[arg(short = 'd', long)]
    drop_mesh: Vec<usize>,
    #[command(flatten)]
    rarg: crate::ReadArgs,
    #[command(flatten)]
    warg: crate::WriteArgs,
    #[command(flatten)]
    oarg: crate::OutputArgs,
    #[command(flatten)]
    paths: crate::InOutPaths,
}

pub fn run(
    _args_common: &CommonArgs,
    args_cmd: &EditArgs,
) -> AnyResult<()> {
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

    let mut infile = std::fs::File::open(&args_cmd.paths.in_file)
        .context("Could not open input file")?;
    let reader = IyesMeshReader::init_with_settings(
        IyesMeshReaderSettings::from(&args_cmd.rarg),
        &mut infile,
    )
    .context("Cannot decode file metadata and initialize decoding")?;
    let with_data =
        reader.read_all_data().context("Cannot decode file data")?;
    let flatbufs =
        with_data.into_flat_buffers().context("Cannot decode file buffers")?;
    let meshes = with_data
        .into_split_meshes(&flatbufs)
        .context("Cannot decode file meshes")?;

    match (args_cmd.drop_user_data, &args_cmd.user_data) {
        (false, None) => {
            if let Some(data) = flatbufs.user_data {
                writer.set_user_data(data);
            } else {
                writer.clear_user_data();
            }
        }
        (true, None) => {
            writer.clear_user_data();
        }
        _ => {}
    }

    let drop_meshes: HashSet<_> = args_cmd.drop_mesh.iter().copied().collect();
    for (i, m) in meshes.meshes.iter().enumerate() {
        if drop_meshes.contains(&i) {
            continue;
        }
        writer.add_mesh(m.clone()).context("Cannot use mesh for output")?;
    }

    let outpath =
        args_cmd.paths.out_file.as_ref().unwrap_or(&args_cmd.paths.in_file);
    let outfile = if args_cmd.oarg.overwrite
        || args_cmd.paths.out_file.is_none()
    {
        std::fs::File::create(outpath).context("Could not open output file")?
    } else {
        std::fs::File::create_new(outpath)
            .context("Could not open output file")?
    };
    let mut bufout = BufWriter::new(outfile);
    writer.write_to(&mut bufout).context("Cannot encode output file")?;
    Ok(())
}
