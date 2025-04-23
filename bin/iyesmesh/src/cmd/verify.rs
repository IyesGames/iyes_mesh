use iyes_mesh::read::IyesMeshReader;
use iyes_mesh::read::IyesMeshReaderSettings;

use crate::CommonArgs;
use crate::prelude::*;

#[derive(clap::Args, Debug)]
pub struct VerifyArgs {
    #[command(flatten)]
    inarg: crate::ReadArgs,
    #[command(flatten)]
    inpath: crate::InputPath,
}

pub fn run(
    args_common: &CommonArgs,
    args_cmd: &VerifyArgs,
) -> AnyResult<()> {
    let mut settings = IyesMeshReaderSettings {
        verify_metadata_checksum: true,
        verify_data_checksum: true,
    };
    if args_cmd.inarg.ignore_checksums {
        if let Err(e) = try_run(args_common, args_cmd, settings) {
            eprintln!("Error! {:#}", e);
            eprintln!("Warning! Trying again without checksum verification.");
            settings.verify_metadata_checksum = false;
            settings.verify_data_checksum = false;
            try_run(args_common, args_cmd, settings)
        } else {
            Ok(())
        }
    } else {
        try_run(args_common, args_cmd, settings)
    }
}

pub fn try_run(
    args_common: &CommonArgs,
    args_cmd: &VerifyArgs,
    settings: IyesMeshReaderSettings,
) -> AnyResult<()> {
    let mut file = std::fs::File::open(&args_cmd.inpath.in_file)
        .context("Could not open input file")?;
    let reader = IyesMeshReader::init_with_settings(settings, &mut file)
        .context("Cannot decode file metadata and initialize decoding")?;
    if args_common.verbose {
        eprintln!("File metadata OK.");
    }
    let with_data = reader.read_all_data()
        .context("Cannot decode file data")?;
    if args_common.verbose {
        eprintln!("File data successfully decoded.");
    }
    let bufs = with_data.into_flat_buffers()
        .context("Cannot parse file data as flat buffers")?;
    if args_common.verbose {
        eprintln!("File data successfully parsed as flat buffers.");
    }
    let _meshes = with_data.into_split_meshes(&bufs)
        .context("Cannot parse file data as split meshes")?;
    if args_common.verbose {
        eprintln!("File data successfully parsed as split meshes.");
    }
    Ok(())
}
