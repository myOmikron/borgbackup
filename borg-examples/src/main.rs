use std::env;
use std::num::NonZeroU16;

use borgbackup::asynchronous::{compact, create, create_progress, list, prune, CreateProgress};
use borgbackup::common::{
    CommonOptions, CompactOptions, CompressionMode, CreateOptions, EncryptionMode, InitOptions,
    ListOptions, PruneOptions,
};
use borgbackup::sync;
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

    let res = sync::init(
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

    let stats = create_progress(
        &CreateOptions {
            repository: repo.clone(),
            archive: "archive 123".to_string(),
            passphrase: Some(passphrase.clone()),
            comment: None,
            compression: Some(CompressionMode::Zstd(3)),
            paths: vec![
                "/home/omikron/git/502-bad-gateway".to_string(),
                "/home/omikron/git/borgbackup".to_string(),
            ],
            exclude_caches: false,
            patterns: vec![],
            excludes: vec![],
            numeric_ids: false,
            sparse: false,
            read_special: false,
            no_xattrs: false,
            no_acls: false,
            no_flags: false,
        },
        &CommonOptions::default(),
        tx,
    )
    .await
    .unwrap();

    let out = list(
        &ListOptions {
            repository: repo.clone(),
            passphrase: Some(passphrase.clone()),
        },
        &CommonOptions::default(),
    )
    .await
    .unwrap();

    info!("{out:#?}");

    prune(
        &PruneOptions {
            repository: repo.clone(),
            passphrase: Some(passphrase.clone()),
            keep_within: None,
            keep_secondly: Some(NonZeroU16::try_from(1).unwrap()),
            keep_minutely: None,
            keep_hourly: None,
            keep_daily: None,
            keep_weekly: None,
            keep_monthly: None,
            keep_yearly: None,
            checkpoint_interval: None,
            glob_archives: None,
        },
        &CommonOptions::default(),
    )
    .await
    .unwrap();

    create(
        &CreateOptions {
            repository: repo.clone(),
            archive: "archive 321".to_string(),
            passphrase: Some(passphrase.clone()),
            comment: None,
            compression: Some(CompressionMode::Zstd(3)),
            paths: vec![
                "/home/omikron/git/502-bad-gateway".to_string(),
                "/home/omikron/git/borgbackup".to_string(),
            ],
            exclude_caches: false,
            patterns: vec![],
            excludes: vec![],
            numeric_ids: false,
            sparse: false,
            read_special: false,
            no_xattrs: false,
            no_acls: false,
            no_flags: false,
        },
        &CommonOptions::default(),
    )
    .await
    .unwrap();

    let out = list(
        &ListOptions {
            repository: repo.clone(),
            passphrase: Some(passphrase.clone()),
        },
        &CommonOptions::default(),
    )
    .await
    .unwrap();

    info!("{out:#?}");

    compact(
        &CompactOptions { repository: repo },
        &CommonOptions::default(),
    )
    .await
    .unwrap();

    println!(
        "Original: {}, Deduplicated: {}",
        Byte::from_bytes(stats.archive.stats.original_size as u128).get_appropriate_unit(true),
        Byte::from_bytes(stats.archive.stats.deduplicated_size as u128).get_appropriate_unit(true)
    );

    dir.close().unwrap();
}
