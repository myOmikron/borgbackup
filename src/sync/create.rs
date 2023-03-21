use std::env;
use std::process::Command;

use log::{debug, info, trace};

use crate::common::{create_fmt_args, create_parse_output, CommonOptions, CreateOptions};
use crate::errors::CreateError;
use crate::output::create::Create;

/// This command creates a backup archive containing all files found
/// while recursively traversing all paths specified.
/// Paths are added to the archive as they are given,
/// that means if relative paths are desired, the command has to be run from the correct directory.
///
/// **Parameter**:
/// - `options`: Reference to [CreateOptions]
/// - `common_options`: Reference to [CommonOptions]
pub fn create(
    options: &CreateOptions,
    common_options: &CommonOptions,
) -> Result<Create, CreateError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = create_fmt_args(options, common_options, false);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CreateError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let stats = create_parse_output(res)?;

    info!("Finished creating archive");

    Ok(stats)
}
