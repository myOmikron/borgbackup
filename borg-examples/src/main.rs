use std::env;

use borgbackup::commands;
use borgbackup::commands::common::{CommonOptions, CompressionMode, EncryptionMode};
use borgbackup::commands::{CreateOptions, CreateProgress, InitOptions};
use byte_unit::Byte;
use log::{error, info};
use tempfile::tempdir;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let dir = tempdir().unwrap();

    let repo = format!("{} 123", dir.path().display());
    let passphrase = "awd".to_string();

    let res = commands::init(
        &InitOptions::new(repo.clone(), EncryptionMode::Repokey(passphrase.clone())),
        &CommonOptions::default(),
    );

    if let Err(err) = res {
        error!("Error running borg init: {err}");
    }

    let (tx, mut rx) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            match progress {
                CreateProgress::Progress {
                    original_size,
                    compressed_size,
                    deduplicated_size,
                    ..
                } => {
                    info!(
                        "O: {} C: {} D: {}",
                        Byte::from_bytes(original_size as u128).get_appropriate_unit(true),
                        Byte::from_bytes(compressed_size as u128).get_appropriate_unit(true),
                        Byte::from_bytes(deduplicated_size as u128).get_appropriate_unit(true)
                    )
                }
                CreateProgress::Finished => {}
            }
        }
    });

    let create = commands::create_async_progress(
        &CreateOptions {
            repository: repo,
            archive: "archive 123".to_string(),
            passphrase: Some(passphrase),
            comment: None,
            compression: Some(CompressionMode::Zstd(3)),
            paths: vec![
                "/home/omikron/git/502-bad-gateway".to_string(),
                "/home/omikron/git/borgbackup".to_string(),
            ],
        },
        &CommonOptions::default(),
        tx,
    )
    .await
    .unwrap();

    println!(
        "Original: {}, Deduplicated: {}",
        byte_unit::Byte::from_bytes(create.archive.stats.original_size as u128)
            .get_appropriate_unit(true),
        byte_unit::Byte::from_bytes(create.archive.stats.deduplicated_size as u128)
            .get_appropriate_unit(true)
    );

    dir.close().unwrap();
}
