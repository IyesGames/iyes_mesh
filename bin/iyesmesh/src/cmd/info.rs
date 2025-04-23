use iyes_mesh::read::IyesMeshReader;
use iyes_mesh::read::IyesMeshReaderSettings;

use crate::CommonArgs;
use crate::prelude::*;

#[derive(clap::Args, Debug)]
pub struct InfoArgs {
    #[command(flatten)]
    rarg: crate::ReadArgs,
    #[command(flatten)]
    inpath: crate::InputPath,
}

pub fn run(
    _args_common: &CommonArgs,
    args_cmd: &InfoArgs,
) -> AnyResult<()> {
    let mut infile = std::fs::File::open(&args_cmd.inpath.in_file)
        .context("Could not open input file")?;
    let reader = IyesMeshReader::init_with_settings(
        IyesMeshReaderSettings::from(&args_cmd.rarg),
        &mut infile,
    )
    .context("Cannot decode file metadata and initialize decoding")?;

    println!("{:#?}", reader.descriptor());

    Ok(())
}
