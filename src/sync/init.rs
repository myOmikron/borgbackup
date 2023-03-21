use std::env;
use std::process::Command;

use log::{debug, info, trace};

use crate::common::{init_fmt_args, init_parse_result, CommonOptions, InitOptions};
use crate::errors::InitError;

/// The entry point for the borg init command
///
/// **Parameter**:
/// - `options`: Reference to [InitOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub fn init(options: &InitOptions, common_options: &CommonOptions) -> Result<(), InitError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = init_fmt_args(options, common_options);
    let passphrase = options.encryption_mode.get_passphrase();

    if let Some(passphrase) = passphrase {
        trace!("Set BORG_PASSPHRASE environment variable");
        env::set_var("BORG_PASSPHRASE", passphrase);
    }

    debug!("Calling borg: {local_path} {args}");

    let args = shlex::split(&args).ok_or(InitError::ShlexError)?;
    let res = Command::new(local_path).args(args).output()?;

    if passphrase.is_some() {
        trace!("Clearing BORG_PASSPHRASE environment variable");
        env::remove_var("BORG_PASSPHRASE");
    }

    init_parse_result(res)?;

    info!("Repository {} created", options.repository);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::common::{CommonOptions, EncryptionMode};
    use crate::sync::init::{init, InitOptions};

    #[test]
    fn create_repo() {
        let dir = tempfile::tempdir().unwrap();

        let pw = String::from("pw");

        let init_options = [
            InitOptions::new(
                format!("{}", dir.path().join("none").display()),
                EncryptionMode::None,
            ),
            InitOptions::new(
                format!("{}", dir.path().join("auth").display()),
                EncryptionMode::Authenticated(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("authb2").display()),
                EncryptionMode::AuthenticatedBlake2(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("repo").display()),
                EncryptionMode::Repokey(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("key").display()),
                EncryptionMode::Keyfile(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("repob2").display()),
                EncryptionMode::RepokeyBlake2(pw.clone()),
            ),
            InitOptions::new(
                format!("{}", dir.path().join("keyb2").display()),
                EncryptionMode::KeyfileBlake2(pw),
            ),
        ];

        for init_options in &init_options {
            assert!(init(init_options, &CommonOptions::default()).is_ok());
        }

        dir.close().unwrap()
    }
}
