use std::io::BufWriter;

use iyes_mesh::read::{IyesMeshReader, IyesMeshReaderSettings};
use iyes_mesh::write::{IyesMeshWriter, IyesMeshWriterSettings};

use crate::CommonArgs;
use crate::prelude::*;
use crate::util::load_user_data;

#[derive(clap::Args, Debug)]
pub struct MergeArgs {
    /// File to load user data from (stdin if unspecified)
    ///
    /// If the file is an IMA file, extract the user data from it.
    /// If the file is not an IMA file, use its raw contents as-is.
    #[arg(short, long)]
    user_data: Option<Option<PathBuf>>,
    /// If a user data file is provided, do not try to parse it as an IMA file
    #[arg(long)]
    user_data_force_raw: bool,
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
    args_cmd: &MergeArgs,
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

    let mut in_data = vec![];
    let mut in_parsed = vec![];

    for inpath in args_cmd.inpaths.in_files.iter() {
        let mut infile =
            std::fs::File::open(inpath).context("Could not open input file")?;
        let reader = IyesMeshReader::init_with_settings(
            IyesMeshReaderSettings::from(&args_cmd.rarg),
            &mut infile,
        )
        .context("Cannot decode file metadata and initialize decoding")?;
        let with_data =
            reader.read_all_data().context("Cannot decode file data")?;
        in_data.push(with_data);
    }

    for with_data in in_data.iter() {
        let flatbufs = with_data
            .into_flat_buffers()
            .context("Cannot decode file buffers")?;
        let meshes = with_data
            .into_split_meshes(&flatbufs)
            .context("Cannot decode file meshes")?;
        in_parsed.push(meshes);
    }

    for src in in_parsed.iter() {
        for m in src.meshes.iter() {
            writer.add_mesh(m.clone()).context("Cannot use mesh for output")?;
        }
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
