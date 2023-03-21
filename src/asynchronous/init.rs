use std::env;

use log::{debug, info, trace};

use crate::common::{init_fmt_args, init_parse_result, CommonOptions, InitOptions};
use crate::errors::InitError;

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [InitOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn init(options: &InitOptions, common_options: &CommonOptions) -> Result<(), InitError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = init_fmt_args(options, common_options);
    let passphrase = options.encryption_mode.get_passphrase();

    if let Some(passphrase) = passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(InitError::ShlexError)?;
    let res = tokio::process::Command::new(local_path)
        .args(args)
        .output()
        .await?;

    if passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    init_parse_result(res)?;

    info!("Repository {} created", options.repository);

    Ok(())
}
