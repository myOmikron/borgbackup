//! The asynchronous version of the borg commands are defined in this module

use std::io;
use std::process::Output;

pub use compact::compact;
pub use create::{create, create_progress, CreateProgress};
pub use extract::extract;
pub use init::init;
pub use list::list;
pub use mount::{mount, umount};
pub use prune::prune;

mod compact;
mod create;
mod extract;
mod init;
mod list;
mod mount;
mod prune;

pub(crate) async fn execute_borg(
    local_path: &str,
    args: Vec<String>,
    passphrase: &Option<String>,
) -> Result<Output, io::Error> {
    Ok(if let Some(passphrase) = passphrase {
        tokio::process::Command::new(local_path)
            .env("BORG_PASSPHRASE", passphrase)
            .args(args)
            .output()
            .await?
    } else {
        tokio::process::Command::new(local_path)
            .args(args)
            .output()
            .await?
    })
}
