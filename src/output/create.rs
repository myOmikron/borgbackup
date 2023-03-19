//! The definitions of the borg create command

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::common::{Cache, Encryption, Limits, Repository};
use crate::output::info::ArchiveStats;

/// The output of a borg create command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Create {
    /// Information about the repository
    repository: Repository,
    /// Information about the cache
    cache: Option<Cache>,
    /// Information about the encryption of the repository
    encryption: Option<Encryption>,
    /// The list of archives with their information
    archives: Vec<CreateArchive>,
}

/// The archive output of a borg create command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateArchive {
    /// Hexadecimal archive ID
    pub id: String,
    /// Name of the archive
    pub name: String,
    /// Array of strings of the command line that created the archive
    ///
    /// The note about paths from above applies here as well.
    pub command_line: Vec<String>,
    /// Object describing the utilization of Borg limits
    pub limits: Limits,
    /// Duration in seconds between start and end in seconds
    pub duration: f64,
    /// The chunker parameters the archive has been created with.
    pub chunker_params: Vec<Value>,
    /// Start timestamp
    pub start: NaiveDateTime,
    /// End timestamp.
    pub end: NaiveDateTime,
    /// The stats of the archive.
    ///
    /// Archive statistics (freshly calculated, this is what makes “info” more expensive)
    pub stats: ArchiveStats,
}
