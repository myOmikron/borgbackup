use log::{debug, info};
use std::env;

use crate::common::{extract_fmt_args, extract_parse_output, CommonOptions, ExtractOptions};
use crate::errors::ExtractError;
use crate::sync::execute_borg;

/// Extract the contents of an archive.
///
/// This command extracts the contents of an archive to the specified destination.
/// By default, the entire archive is extracted but you can use the strip_components
/// option to remove leading path elements.
///
/// **Parameter**:
/// - `options`: Reference to [ExtractOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn extract(
    options: &ExtractOptions,
    common_options: &CommonOptions,
) -> Result<(), ExtractError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    // Change to the destination directory before executing borg extract
    let original_dir = env::current_dir().map_err(|e| {
        ExtractError::CommandFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get current directory: {}", e),
        ))
    })?;

    env::set_current_dir(&options.destination).map_err(|e| {
        ExtractError::CommandFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to change to destination directory: {}", e),
        ))
    })?;

    let args = extract_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ExtractError::ShlexError)?;
    let res = execute_borg(local_path, args, &options.passphrase);

    // Restore original directory
    let _ = env::set_current_dir(original_dir);

    let res = res?;
    extract_parse_output(res)?;

    info!("Finished extracting archive");

    Ok(())
}
