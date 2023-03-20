//! The common options of borg commands are defined here

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

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
