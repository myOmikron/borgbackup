//! Common types that are used throughout the API are defined in this module

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Information about the repository
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    /// The ID of the repository, normally 64 hex characters
    pub id: String,
    /// Canonical repository path, thus this may be different
    /// from what is specified on the command line
    pub location: String,
    /// Date when the repository was last modified by the Borg client
    pub last_modified: NaiveDateTime,
}

/// The encryption settings of the repository
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Encryption {
    /// The selected encryption mode
    pub mode: EncryptionMode,
    /// Path to the local key file used for access.
    ///
    /// Depending on mode this key may be absent.
    pub keyfile: Option<String>,
}

/// The encryption mode of the repository.
///
/// See <https://borgbackup.readthedocs.io/en/stable/usage/init.html#more-encryption-modes>
/// for further information about encryption modes.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum EncryptionMode {
    /// No encryption, nor hashing.
    ///
    /// This mode is not recommended.
    #[serde(rename = "none")]
    None,
    /// Uses no encryption, but authenticates repository contents through HMAC-SHA256 hashes
    #[serde(rename = "authenticated")]
    Authenticated,
    /// Uses no encryption, but authenticates repository contents through keyed BLAKE2b-256 hashes
    #[serde(rename = "authenticated-blake2")]
    AuthenticatedBlake2,
    /// Use AES-CTR-256 for encryption and HMAC-SHA256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is HMAC-SHA256 as well (with a separate key).
    ///
    /// Stores the key in the repository.
    #[serde(rename = "repokey")]
    Repokey,
    /// Use AES-CTR-256 for encryption and HMAC-SHA256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is HMAC-SHA256 as well (with a separate key).
    ///
    /// Stores the key locally.
    #[serde(rename = "keyfile")]
    Keyfile,
    /// Use AES-CTR-256 for encryption and BLAKE2b-256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is a keyed BLAKE2b-256 hash.
    ///
    /// Stores the key in the repository.
    #[serde(rename = "repokey-blake2")]
    RepokeyBlake2,
    /// Use AES-CTR-256 for encryption and BLAKE2b-256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is a keyed BLAKE2b-256 hash.
    ///
    /// Stores the key locally.
    #[serde(rename = "keyfile-blake2")]
    KeyfileBlake2,
}

/// The cache info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cache {
    /// Path to the local repository cache
    pub path: String,
    /// Stats of the cache
    pub stats: CacheStats,
}

/// The stats of the cache
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of chunks
    pub total_chunks: u64,
    /// Total compressed and encrypted size of all chunks multiplied with their reference counts
    pub total_csize: u64,
    /// Total uncompressed size of all chunks multiplied with their reference counts
    pub total_size: u64,
    /// Number of unique chunks
    pub total_unique_chunks: u64,
    /// Compressed and encrypted size of all chunks
    pub unique_csize: u64,
    /// Uncompressed size of all chunks
    pub unique_size: u64,
}

/// Object describing the utilization of Borg limits
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Limits {
    /// Float between 0 and 1 describing how large this archive is relative
    /// to the maximum size allowed by Borg
    pub max_archive_size: f64,
}
