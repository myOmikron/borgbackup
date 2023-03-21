use std::env;

use log::{debug, info, trace};

use crate::common::{prune_fmt_args, prune_parse_output, CommonOptions, PruneOptions};
use crate::errors::PruneError;

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [PruneOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn prune(
    options: &PruneOptions,
    common_options: &CommonOptions,
) -> Result<(), PruneError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = prune_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(PruneError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    prune_parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}
