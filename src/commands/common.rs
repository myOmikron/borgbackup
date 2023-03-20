//! The common options of borg commands are defined here

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// A pattern instruction.
/// These instructions will be used for the `--pattern` command line parameter.
///
/// [PatternInstruction::Include] are useful to include paths that are contained in an excluded path.
/// The first matching pattern is used so if an [PatternInstruction::Include] matches before
/// an [PatternInstruction::Exclude], the file is backed up.
/// If an [PatternInstruction::ExcludeNoRecurse] pattern matches a directory,
/// it won't recurse into it and won't discover any potential matches for include rules
/// below that directory.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PatternInstruction {
    /// A plain root path to use as a starting point
    Root(String),
    /// Include matched files
    Include(Pattern),
    /// Exclude matched files
    Exclude(Pattern),
    /// If an [PatternInstruction::ExcludeNoRecurse] pattern matches a directory, it won't recurse into
    /// it and won't discover any potential matches for include rules below that directory.
    ExcludeNoRecurse(Pattern),
}

impl Display for PatternInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternInstruction::Root(path) => write!(f, "P {path}"),
            PatternInstruction::Include(pattern) => write!(f, "+ {pattern}"),
            PatternInstruction::Exclude(pattern) => write!(f, "- {pattern}"),
            PatternInstruction::ExcludeNoRecurse(pattern) => write!(f, "! {pattern}"),
        }
    }
}

/// The path/filenames used as input for the pattern matching start from the
/// currently active recursion root. You usually give the recursion root(s)
/// when invoking borg and these can be either relative or absolute paths.
///
/// [Pattern::Regex], [Pattern::Shell] and [Pattern::FnMatch] patterns are all implemented on top
/// of the Python SRE engine. It is very easy to formulate patterns for each of these types which
/// requires an inordinate amount of time to match paths.
///
/// If untrusted users are able to supply patterns, ensure they cannot supply [Pattern::Regex]
/// patterns. Further, ensure that [Pattern::Shell] and [Pattern::FnMatch] patterns only contain a
/// handful of wildcards at most.
///
/// Note that this enum is only for use in [PatternInstruction].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Pattern {
    /// Fnmatch <https://docs.python.org/3/library/fnmatch.html>:
    ///
    /// These patterns use a variant of shell pattern syntax, with '\*' matching
    /// any number of characters, '?' matching any single character, '\[...]'
    /// matching any single character specified, including ranges, and '\[!...]'
    /// matching any character not specified.
    ///
    /// For the purpose of these patterns, the path separator
    /// (backslash for Windows and '/' on other systems) is not treated specially.
    /// Wrap meta-characters in brackets for a literal match
    /// (i.e. \[?] to match the literal character ?).
    /// For a path to match a pattern, the full path must match, or it must match
    /// from the start of the full path to just before a path separator. Except
    /// for the root path, paths will never end in the path separator when
    /// matching is attempted.  Thus, if a given pattern ends in a path
    /// separator, a '*' is appended before matching is attempted.
    ///
    /// A leading path separator is always removed.
    FnMatch(String),
    /// Like [Pattern::FnMatch] patterns these are similar to shell patterns.
    /// The difference is that the pattern may include **/ for matching zero or more
    /// directory levels, * for matching zero or more arbitrary characters with the exception of
    /// any path separator.
    ///
    /// A leading path separator is always removed.
    Shell(String),
    /// Regular expressions similar to those found in Perl are supported. Unlike shell patterns,
    /// regular expressions are not required to match the full path and any substring match
    /// is sufficient.
    ///
    /// It is strongly recommended to anchor patterns to the start ('^'), to the end ('$') or both.
    ///
    /// Path separators (backslash for Windows and '/' on other systems) in paths are
    /// always normalized to a forward slash ('/') before applying a pattern.
    ///
    /// The regular expression syntax is described in the Python documentation for the re module
    /// <https://docs.python.org/3/library/re.html>.
    Regex(String),
    /// This pattern style is useful to match whole sub-directories.
    ///
    /// The pattern `root/somedir` matches `root/somedir` and everything therein.
    /// A leading path separator is always removed.
    PathPrefix(String),
    /// This pattern style is (only) useful to match full paths.
    ///
    /// This is kind of a pseudo pattern as it can not have any variable or unspecified parts -
    /// the full path must be given. `root/file.ext` matches `root/file.ext` only.
    ///
    /// A leading path separator is always removed.
    ///
    /// Implementation note: this is implemented via very time-efficient O(1)
    /// hashtable lookups (this means you can have huge amounts of such patterns
    /// without impacting performance much).
    /// Due to that, this kind of pattern does not respect any context or order.
    /// If you use such a pattern to include a file, it will always be included
    /// (if the directory recursion encounters it).
    /// Other include/exclude patterns that would normally match will be ignored.
    /// Same logic applies for exclude.
    PathFullMatch(String),
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::FnMatch(x) => write!(f, "fm:{x}"),
            Pattern::Shell(x) => write!(f, "sh:{x}"),
            Pattern::Regex(x) => write!(f, "re:{x}"),
            Pattern::PathPrefix(x) => write!(f, "pp:{x}"),
            Pattern::PathFullMatch(x) => write!(f, "pf:{x}"),
        }
    }
}

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
/// See <https://borgbackup.readthedocs.io/en/stable/usage/init.html#more-encryption-modes>
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
