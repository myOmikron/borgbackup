//! The logging definitions

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All valid logging messages are defined here
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LoggingMessage {
    /// Any regular log output invokes this type.
    /// Regular log options and filtering applies to these as well.
    #[serde(rename = "log_message")]
    LogMessage {
        /// Unix timestamp
        time: f64,
        /// The loglevel of borg
        #[serde(rename = "levelname")]
        level_name: LevelName,
        /// Name of the emitting entity
        name: String,
        /// Formatted log message
        message: String,
        /// Message ID.
        ///
        /// Specifies more information or error information
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
    },
    /// This is only output by borg create and borg recreate if --list is specified.
    /// The usual rules for the file listing applies, including the --filter option.
    #[serde(rename = "file_status")]
    FileStatus {
        /// Single-character status as for regular list output
        status: String,
        /// Path of the file system object
        path: String,
    },
    /// Absolute progress information with defined end/total and current value.
    #[serde(rename = "progress_percent")]
    ProgressPercent {
        /// unique, opaque integer ID of the operation
        operation: u64,
        /// Message ID of the operation
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
        /// Unix timestamp
        time: f64,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
        /// Current value
        ///
        /// absent for finished == true
        current: Option<u64>,
        /// Total value
        ///
        /// absent for finished == true
        total: Option<u64>,
        /// Array that describes the current item, may be null, contents depend on msg_id
        ///
        /// absent for finished == true
        info: Option<Vec<Value>>,
    },
    /// A message-based progress information with no concrete progress information,
    /// just a message saying what is currently being worked on.
    #[serde(rename = "progress_message")]
    ProgressMessage {
        /// unique, opaque integer ID of the operation
        operation: u64,
        /// Message ID of the operation
        #[serde(rename = "msgid")]
        msg_id: Option<MessageId>,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
        /// current progress message (may be empty/absent)
        message: Option<String>,
        /// Unix timestamp
        time: f64,
    },
    /// Output during operations creating archives (borg create and borg recreate)
    #[serde(rename = "archive_progress")]
    ArchiveProgress {
        /// Original size of data processed so far
        /// before compression and deduplication, may be empty/absent
        original_size: Option<u64>,
        /// Compressed size (may be empty/absent)
        compressed_size: Option<u64>,
        /// Deduplicated size (may be empty/absent)
        deduplicated_size: Option<u64>,
        /// Number of (regular) files processed so far (may be empty/absent)
        nfiles: Option<u64>,
        /// Current path (may be empty/absent)
        path: Option<String>,
        /// Unix timestamp
        time: f64,
        /// Indicating whether the operation has finished,
        /// only the last object for an operation can have this property set to true.
        finished: bool,
    },
}

/// The valid loglevel of borg
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LevelName {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Message IDs are strings that essentially give a log message or operation a name,
/// without actually using the full text, since texts change more frequently.
///
/// Message IDs are unambiguous and reduce the need to parse log messages.
#[derive(Deserialize, Serialize)]
pub enum MessageId {
    /// Archive {} already exists
    #[serde(rename = "Archive.AlreadyExists")]
    ArchiveAlreadyExists,
    /// Archive {} does not exist
    #[serde(rename = "Archive.DoesNotExist")]
    ArchiveDoesNotExist,
    /// Failed to encode filename “{}” into file system encoding “{}”.
    /// Consider configuring the LANG environment variable.
    #[serde(rename = "Archive.IncompatibleFilesystemEncodingError")]
    ArchiveIncompatibleFilesystemEncodingError,
    /// Cache initialization aborted
    #[serde(rename = "Cache.CacheInitAbortedError")]
    CacheCacheInitAbortedError,
    /// Repository encryption method changed since last access, refusing to continue
    #[serde(rename = "Cache.EncryptionMethodMismatch")]
    CacheEncryptionMethodMismatch,
    /// Repository access aborted
    #[serde(rename = "Cache.RepositoryAccessAborted")]
    CacheRepositoryAccessAborted,
    /// Cache is newer than repository - do you have multiple,
    /// independently updated repos with same ID?
    #[serde(rename = "Cache.RepositoryIDNotUnique")]
    CacheRepositoryIDNotUnique,
    /// Cache is newer than repository - this is either an attack
    /// or unsafe (multiple repos with same ID)
    #[serde(rename = "Cache.RepositoryReplay")]
    CacheRepositoryReplay,
    /// Requested buffer size {} is above the limit of {}.
    #[serde(rename = "Buffer.MemoryLimitExceeded")]
    BufferMemoryLimitExceeded,
    /// The Borg binary extension modules do not seem to be properly installed
    ExtensionModuleError,
    /// Data integrity error: {}
    IntegrityError,
    /// Repository has no manifest.
    NoManifestError,
    /// Formatting Error: “{}”.format({}): {}({})
    PlaceholderError,
    /// Invalid key file for repository {} found in {}.
    KeyfileInvalidError,
    /// Mismatch between repository {} and key file {}.
    KeyfileMismatchError,
    /// No key file for repository {} found in {}.
    KeyfileNotFoundError,
    /// passphrase supplied in BORG_PASSPHRASE is incorrect
    PassphraseWrong,
    /// exceeded the maximum password retries
    PasswordRetriesExceeded,
    /// No key entry found in the config of repository {}.
    RepoKeyNotFoundError,
    /// Unsupported manifest envelope. A newer version is required to access this repository.
    UnsupportedManifestError,
    /// Unsupported payload type {}. A newer version is required to access this repository.
    UnsupportedPayloadError,
    /// This file is not a borg key backup, aborting.
    NotABorgKeyFile,
    /// This key backup seems to be for a different backup repository, aborting.
    RepoIdMismatch,
    /// Key-management not available for unencrypted repositories.
    UnencryptedRepo,
    /// Keytype {0} is unknown.
    UnknownKeyType,
    /// Failed to acquire the lock {}.
    LockError,
    /// Failed to acquire the lock {}.
    LockErrorT,
    /// Connection closed by remote host
    ConnectionClosed,
    /// RPC method {} is not valid
    InvalidRPCMethod,
    /// Repository path not allowed
    PathNotAllowed,
    /// Borg server is too old for {}. Required version {}
    #[serde(rename = "RemoteRepository.RPCServerOutdated")]
    RemoteRepositoryRPCServerOutdated,
    /// Borg {}: Got unexpected RPC data format from client.
    UnexpectedRPCDataFormatFromClient,
    /// Got unexpected RPC data format from server: {}
    UnexpectedRPCDataFormatFromServer,
    /// Repository {} already exists.
    #[serde(rename = "Repository.AlreadyExists")]
    RepositoryAlreadyExists,
    /// Inconsistency detected. Please run “borg check {}”.
    #[serde(rename = "Repository.CheckNeeded")]
    RepositoryCheckNeeded,
    /// Repository {} does not exist.
    #[serde(rename = "Repository.DoesNotExist")]
    RepositoryDoesNotExist,
    /// Insufficient free space to complete transaction (required: {}, available: {}).
    #[serde(rename = "Repository.InsufficientFreeSpaceError")]
    RepositoryInsufficientFreeSpaceError,
    /// {} is not a valid repository. Check repo config.
    #[serde(rename = "Repository.InvalidRepository")]
    RepositoryInvalidRepository,
    /// Attic repository detected. Please run “borg upgrade {}”.
    #[serde(rename = "Repository.AtticRepository")]
    RepositoryAtticRepository,
    /// Object with key {} not found in repository {}.
    #[serde(rename = "Repository.ObjectNotFound")]
    RepositoryObjectNotFound,
}
