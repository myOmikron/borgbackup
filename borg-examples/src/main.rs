use std::env;

use borgbackup::commands;
use borgbackup::commands::common::{CommonOptions, EncryptionMode};
use borgbackup::commands::InitOptions;
use log::error;
use tempfile::tempdir;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let dir = tempdir().unwrap();

    let res = commands::init(
        &InitOptions::new(
            format!("{} 123", dir.path().display()),
            EncryptionMode::Repokey("awd".to_string()),
        ),
        &CommonOptions::default(),
    );

    if let Err(err) = res {
        error!("Error running borg init: {err}");
    }

    let res = commands::init(
        &InitOptions::new(
            format!("{} 123", dir.path().display()),
            EncryptionMode::Repokey("awd".to_string()),
        ),
        &CommonOptions::default(),
    );

    if let Err(err) = res {
        error!("Error running borg init: {err}");
    }

    dir.close().unwrap();
}
