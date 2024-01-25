use log::{debug, info};

use crate::common::{list_fmt_args, list_parse_output, CommonOptions, ListOptions};
use crate::errors::ListError;
use crate::output::list::ListRepository;
use crate::sync::execute_borg;

/// The entry point for the borg list command
///
/// **Parameter**:
/// - `options`: Reference to [ListOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn list(
    options: &ListOptions,
    common_options: &CommonOptions,
) -> Result<ListRepository, ListError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = list_fmt_args(options, common_options).map_err(|_| ListError::ShlexError)?;
    debug!("Calling borg: {local_path} {args}");

    let args = shlex::split(&args).ok_or(ListError::ShlexError)?;
    let res = execute_borg(local_path, args, &options.passphrase)?;

    let list_output = list_parse_output(res)?;

    info!("Finished listing repository");

    Ok(list_output)
}
