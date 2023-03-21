use std::process::Command;

use log::{debug, info};

use crate::common::{compact_fmt_args, compact_parse_output, CommonOptions, CompactOptions};
use crate::errors::CompactError;

///This command frees repository space by compacting segments.
///
/// Use this regularly to avoid running out of space - you do not need to use this after each borg
/// command though. It is especially useful after deleting archives,
/// because only compaction will really free repository space.
///
/// compact does not need a key, so it is possible to
/// invoke it from the client or also from the server.
///
/// **Parameter**:
/// - `options`: Reference to [CompactOptions]
/// - `common_options`: Reference to [CommonOptions]
pub fn compact(
    options: &CompactOptions,
    common_options: &CommonOptions,
) -> Result<(), CompactError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = compact_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(CompactError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    compact_parse_output(res)?;

    info!("Finished compacting");

    Ok(())
}
