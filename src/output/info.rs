//! The definition of the info command output

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::common::{Cache, Encryption, Limits, Repository};

/// The output of a borg info call.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Info {
    /// The info of a repository
    Repository {
        /// Information about the repository
        repository: Repository,
        /// Information about the cache
        cache: Option<Cache>,
        /// Information about the encryption of the repository
        encryption: Option<Encryption>,
        /// The security directory
        security_dir: Option<String>,
    },
    /// The info of archives
    Archives {
        /// Information about the repository
        repository: Repository,
        /// Information about the cache
        cache: Option<Cache>,
        /// Information about the encryption of the repository
        encryption: Option<Encryption>,
        /// The list of archives with their information
        archives: Vec<InfoArchive>,
    },
}

/// Information about an archive
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfoArchive {
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
    /// Hostname of the creating host
    pub hostname: String,
    /// Name of the creating user
    pub username: String,
    /// Archive comment, if any
    pub comment: String,
}

/// The stats of an archive
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ArchiveStats {
    /// Size after compression
    pub compressed_size: u64,
    /// Deduplicated size (against the current repository, not when the archive was created)
    pub deduplicated_size: u64,
    /// Number of regular files in the archive
    pub nfiles: u64,
    /// Size of files and metadata before compression
    pub original_size: u64,
}
