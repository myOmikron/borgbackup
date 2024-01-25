use log::{debug, info};

use crate::asynchronous::execute_borg;
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

    let args = prune_fmt_args(options, common_options).map_err(|_| PruneError::ShlexError)?;
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(PruneError::ShlexError)?;
    let res = execute_borg(local_path, args, &options.passphrase).await?;

    prune_parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}
