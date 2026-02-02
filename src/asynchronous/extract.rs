use log::{debug, info};
use std::env;

use crate::asynchronous::execute_borg;
use crate::common::{extract_fmt_args, extract_parse_output, CommonOptions, ExtractOptions};
use crate::errors::ExtractError;

/// Extract the contents of an archive.
///
/// This command extracts the contents of an archive to the specified destination.
/// By default, the entire archive is extracted but you can use the strip_components
/// option to remove leading path elements.
///
/// **Parameter**:
/// - `options`: Reference to [ExtractOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn extract(
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

    // Execute extraction, capturing result before cleanup
    let result = extract_in_destination(local_path, options, common_options).await;

    // Always restore original directory
    env::set_current_dir(original_dir).map_err(|e| {
        ExtractError::CommandFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to restore original directory: {}", e),
        ))
    })?;

    result?;

    info!("Finished extracting archive");

    Ok(())
}

async fn extract_in_destination(
    local_path: &str,
    options: &ExtractOptions,
    common_options: &CommonOptions,
) -> Result<(), ExtractError> {
    env::set_current_dir(&options.destination).map_err(|e| {
        ExtractError::CommandFailed(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to change to destination directory: {}", e),
        ))
    })?;

    let args = extract_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(ExtractError::ShlexError)?;
    let res = execute_borg(local_path, args, &options.passphrase).await?;
    extract_parse_output(res)?;

    Ok(())
}
