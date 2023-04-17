//! Output from the borg list command

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::output::common::{Encryption, Repository};

/// Output of the borg list command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListRepository {
    /// Information about the repository
    pub repository: Repository,
    /// Information about the encryption of the repository
    pub encryption: Option<Encryption>,
    /// Information about the archives in the repository
    pub archives: Vec<ListArchive>,
}

/// The short output version of the archive
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListArchive {
    /// Hexadecimal archive ID
    pub id: String,
    /// Name of the archive
    pub name: String,
    /// Start timestamp
    pub start: NaiveDateTime,
}
