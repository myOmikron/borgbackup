use std::env;

use log::{debug, info, trace};

use crate::common::{list_fmt_args, list_parse_output, CommonOptions, ListOptions};
use crate::errors::ListError;
use crate::output::list::ListRepository;

/// The entry point for the borg list command
///
/// **Parameter**:
/// - `options`: Reference to [ListOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn list(
    options: &ListOptions,
    common_options: &CommonOptions,
) -> Result<ListRepository, ListError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    if let Some(passphrase) = &options.passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    let args = list_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ListError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if options.passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    let list_repo = list_parse_output(res)?;

    info!("Finished listing repository");

    Ok(list_repo)
}
