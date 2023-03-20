//! The common options of borg commands are defined here

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// The compression modes of an archive.
///
/// The compression modes `auto` and `obfuscate` are currently not supported by this library.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum CompressionMode {
    /// No compression
    None,
    /// Use lz4 compression. Very high speed, very low compression.
    Lz4,
    /// Use zstd ("zstandard") compression, a modern wide-range algorithm.
    ///
    /// Allowed values ranging from 1 to 22, where 1 is the fastest and 22 the highest compressing.
    ///
    /// Archives compressed with zstd are not compatible with borg < 1.1.4.
    Zstd(u8),
    /// Use zlib ("gz") compression. Medium speed, medium compression.
    ///
    /// Allowed values ranging from 0 to 9.
    /// Giving level 0 (means "no compression", but still has zlib protocol overhead)
    /// is usually pointless, you better use [CompressionMode::None] compression.
    Zlib(u8),
    /// Use lzma ("xz") compression. Low speed, high compression.
    ///
    /// Allowed values ranging from 0 to 9.
    /// Giving levels above 6 is pointless and counterproductive because it does
    /// not compress better due to the buffer size used by borg - but it wastes
    /// lots of CPU cycles and RAM.
    Lzma(u8),
}

impl Display for CompressionMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionMode::None => write!(f, "none"),
            CompressionMode::Lz4 => write!(f, "lz4"),
            CompressionMode::Zstd(x) => write!(f, "zstd,{x}"),
            CompressionMode::Zlib(x) => write!(f, "zlib,{x}"),
            CompressionMode::Lzma(x) => write!(f, "lzma,{x}"),
        }
    }
}

/// The encryption mode of the repository.
///
/// See https://borgbackup.readthedocs.io/en/stable/usage/init.html#more-encryption-modes
/// for further information about encryption modes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EncryptionMode {
    /// No encryption, nor hashing.
    ///
    /// This mode is not recommended.
    None,
    /// Uses no encryption, but authenticates repository contents through HMAC-SHA256 hashes
    Authenticated(String),
    /// Uses no encryption, but authenticates repository contents through keyed BLAKE2b-256 hashes
    AuthenticatedBlake2(String),
    /// Use AES-CTR-256 for encryption and HMAC-SHA256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is HMAC-SHA256 as well (with a separate key).
    ///
    /// Stores the key in the repository.
    Repokey(String),
    /// Use AES-CTR-256 for encryption and HMAC-SHA256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is HMAC-SHA256 as well (with a separate key).
    ///
    /// Stores the key locally.
    Keyfile(String),
    /// Use AES-CTR-256 for encryption and BLAKE2b-256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is a keyed BLAKE2b-256 hash.
    ///
    /// Stores the key in the repository.
    RepokeyBlake2(String),
    /// Use AES-CTR-256 for encryption and BLAKE2b-256 for authentication in an
    /// encrypt-then-MAC (EtM) construction.
    ///
    /// The chunk ID hash is a keyed BLAKE2b-256 hash.
    ///
    /// Stores the key locally.
    KeyfileBlake2(String),
}

impl EncryptionMode {
    pub(crate) fn get_passphrase(&self) -> Option<&str> {
        match self {
            EncryptionMode::None => None,
            EncryptionMode::Authenticated(x) => Some(x),
            EncryptionMode::AuthenticatedBlake2(x) => Some(x),
            EncryptionMode::Repokey(x) => Some(x),
            EncryptionMode::Keyfile(x) => Some(x),
            EncryptionMode::RepokeyBlake2(x) => Some(x),
            EncryptionMode::KeyfileBlake2(x) => Some(x),
        }
    }
}

impl Display for EncryptionMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionMode::None => write!(f, "none"),
            EncryptionMode::Authenticated(_) => write!(f, "authenticated"),
            EncryptionMode::AuthenticatedBlake2(_) => write!(f, "authenticated-blake2"),
            EncryptionMode::Repokey(_) => write!(f, "repokey"),
            EncryptionMode::Keyfile(_) => write!(f, "keyfile"),
            EncryptionMode::RepokeyBlake2(_) => write!(f, "repokey-blake2"),
            EncryptionMode::KeyfileBlake2(_) => write!(f, "keyfile-blake2"),
        }
    }
}

/// The common options that can be used for every borg command
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct CommonOptions {
    /// The local path to the borg executable. (default = "borg")
    pub local_path: Option<String>,
    /// The remote path to the borg executable. (default = "borg")
    pub remote_path: Option<String>,
    /// set network upload rate limit in kiByte/s (0 = unlimited)
    pub upload_ratelimit: Option<u64>,
    /// Use this command to connect to the ‘borg serve’ process (default: "ssh")
    ///
    /// This can be useful to specify an alternative ssh key: "ssh -i /path/to/privkey"
    pub rsh: Option<String>,
}

impl Display for CommonOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(rsh) = &self.rsh {
            write!(f, "--rsh {} ", shlex::quote(rsh))?;
        }

        if let Some(remote_path) = &self.remote_path {
            write!(f, "--remote-path {} ", shlex::quote(remote_path))?;
        }

        if let Some(upload_ratelimit) = &self.upload_ratelimit {
            write!(f, "--upload-ratelimit {upload_ratelimit} ")?;
        }

        Ok(())
    }
}
