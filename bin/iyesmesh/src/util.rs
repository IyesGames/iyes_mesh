use std::io::Read;

use iyes_mesh::read::{is_iyes_mesh_file, IyesMeshReader, IyesMeshReaderSettings};

use crate::prelude::*;

pub fn load_user_data(
    src: Option<&Path>,
    settings: IyesMeshReaderSettings,
    force_raw_file: bool,
) -> AnyResult<Vec<u8>> {
    let mut new_user_data = vec![];
    match &src {
        None => {
            std::io::stdin()
                .lock()
                .read_to_end(&mut new_user_data)
                .context("Could not read user data from stdin")?;
        }
        Some(path) => {
            let mut udfile = std::fs::File::open(path)
                .context("Could not open user data file")?;
            if !force_raw_file && is_iyes_mesh_file(&mut udfile)
                .context("Cannot autodetect file format")?
            {
                new_user_data = IyesMeshReader::init_with_settings(
                    settings,
                    &mut udfile,
                )
                .and_then(|r| r.read_user_data())
                .context("Cannot extract user data from user data IMA file")?;
            } else {
                udfile.read_to_end(&mut new_user_data)
                .context("Could not read user data from raw file")?;
            }
        }
    }
    Ok(new_user_data)
}
