//! The synchronous versions of the borg command are defined in this module

use std::io;
use std::process::{Command, Output};

pub use compact::compact;
pub use create::create;
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

pub(crate) fn execute_borg(
    local_path: &str,
    args: Vec<String>,
    passphrase: &Option<String>,
) -> Result<Output, io::Error> {
    Ok(if let Some(passphrase) = passphrase {
        Command::new(local_path)
            .env("BORG_PASSPHRASE", passphrase)
            .args(args)
            .output()?
    } else {
        Command::new(local_path).args(args).output()?
    })
}
