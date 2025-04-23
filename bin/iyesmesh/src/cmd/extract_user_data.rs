use std::io::Write;

use iyes_mesh::read::IyesMeshReader;
use iyes_mesh::read::IyesMeshReaderSettings;

use crate::CommonArgs;
use crate::prelude::*;

#[derive(clap::Args, Debug)]
pub struct ExtractUserDataArgs {
    #[command(flatten)]
    rarg: crate::ReadArgs,
    #[command(flatten)]
    oarg: crate::OutputArgs,
    #[command(flatten)]
    inpath: crate::InputPath,
    #[command(flatten)]
    outpath: crate::OptOutputPath,
}

pub fn run(
    _args_common: &CommonArgs,
    args_cmd: &ExtractUserDataArgs,
) -> AnyResult<()> {
    let mut infile = std::fs::File::open(&args_cmd.inpath.in_file)
        .context("Could not open input file")?;
    let reader = IyesMeshReader::init_with_settings(
        IyesMeshReaderSettings::from(&args_cmd.rarg),
        &mut infile,
    )
    .context("Cannot decode file metadata and initialize decoding")?;
    let userdata = reader.read_user_data()
        .context("Cannot decode user data")?;
    if let Some(outpath) = &args_cmd.outpath.out_file {
        let mut outfile = if args_cmd.oarg.overwrite {
            std::fs::File::create(outpath)
                .context("Could not open output file")?
        } else {
            std::fs::File::create_new(outpath)
                .context("Could not open output file")?
        };
        outfile.write_all(&userdata)
            .and_then(|_| outfile.flush())
            .and_then(|_| outfile.sync_all())
            .context("Could not write output")?;
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(&userdata)
            .and_then(|_| stdout.flush())
            .context("Could not write output")?;
    }
    Ok(())
}
