//! The common options of borg commands are defined here

use std::fmt::{Display, Formatter, Write};
use std::io::BufRead;
use std::num::NonZeroU16;
use std::process::Output;

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::errors::{CompactError, CreateError, InitError, ListError, MountError, PruneError};
use crate::output::create::Create;
use crate::output::list::ListRepository;
use crate::output::logging::{LevelName, LoggingMessage, MessageId};

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
    pub(crate) fn get_passphrase(&self) -> Option<String> {
        match self {
            EncryptionMode::None => None,
            EncryptionMode::Authenticated(x) => Some(x.clone()),
            EncryptionMode::AuthenticatedBlake2(x) => Some(x.clone()),
            EncryptionMode::Repokey(x) => Some(x.clone()),
            EncryptionMode::Keyfile(x) => Some(x.clone()),
            EncryptionMode::RepokeyBlake2(x) => Some(x.clone()),
            EncryptionMode::KeyfileBlake2(x) => Some(x.clone()),
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

/// The quantifier for [PruneWithin]
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum PruneWithinTime {
    /// Hour quantifier
    Hour,
    /// Day quantifier
    Day,
    /// Week quantifier (7 days)
    Week,
    /// Month quantifier (31 days)
    Month,
    /// Year quantifier
    Year,
}

impl Display for PruneWithinTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PruneWithinTime::Hour => write!(f, "H"),
            PruneWithinTime::Day => write!(f, "d"),
            PruneWithinTime::Week => write!(f, "w"),
            PruneWithinTime::Month => write!(f, "m"),
            PruneWithinTime::Year => write!(f, "y"),
        }
    }
}

/// The definition to specify an interval in which backups
/// should not be pruned.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct PruneWithin {
    /// quantifier
    pub quantifier: NonZeroU16,
    /// The time quantifier
    pub time: PruneWithinTime,
}

impl Display for PruneWithin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.quantifier, self.time)
    }
}

/// Options for [crate::sync::prune]
///
/// A good procedure is to thin out more and more the older your backups get.
/// As an example, `keep_daily` 7 means to keep the latest backup on each day, up to 7 most
/// recent days with backups (days without backups do not count). The rules are applied from
/// secondly to yearly, and backups selected by previous rules do not count towards those of
/// later rules. The time that each backup starts is used for pruning purposes.
/// Dates and times are interpreted in the local timezone, and weeks go from Monday to Sunday.
/// Specifying a negative number of archives to keep means that there is no limit.
///
/// As of borg 1.2.0, borg will retain the oldest archive if any of the secondly, minutely,
/// hourly, daily, weekly, monthly, or yearly rules was not otherwise able to meet
/// its retention target. This enables the first chronological archive to continue aging until
/// it is replaced by a newer archive that meets the retention criteria.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PruneOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
    /// The archives kept with this option do not count towards the totals specified
    /// by any other options.
    pub keep_within: Option<PruneWithin>,
    /// number of secondly archives to keep
    pub keep_secondly: Option<NonZeroU16>,
    /// number of minutely archives to keep
    pub keep_minutely: Option<NonZeroU16>,
    /// number of hourly archives to keep
    pub keep_hourly: Option<NonZeroU16>,
    /// number of daily archives to keep
    pub keep_daily: Option<NonZeroU16>,
    /// number of weekly archives to keep
    pub keep_weekly: Option<NonZeroU16>,
    /// number of monthly archives to keep
    pub keep_monthly: Option<NonZeroU16>,
    /// number of yearly archives to keep
    pub keep_yearly: Option<NonZeroU16>,
    /// write checkpoint every SECONDS seconds (Default: 1800)
    pub checkpoint_interval: Option<NonZeroU16>,
    /// only consider archive names matching the glob.
    ///
    /// The pattern can use [Pattern::Shell]
    pub glob_archives: Option<String>,
}

impl PruneOptions {
    /// Create an new [PruneOptions]
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            passphrase: None,
            keep_within: None,
            keep_secondly: None,
            keep_minutely: None,
            keep_hourly: None,
            keep_daily: None,
            keep_weekly: None,
            keep_monthly: None,
            keep_yearly: None,
            checkpoint_interval: None,
            glob_archives: None,
        }
    }
}

/// Options for [crate::sync::mount]
///
/// Mount an archive or repository as a FUSE filesystem. This is useful for
/// browsing archives or repositories and interactive restoration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MountSource {
    /// Mount a repository
    Repository {
        /// Name of the repository you wish to mount
        ///
        /// Example values:
        /// - `/tmp/foo`
        /// - `user@example.com:/opt/repo`
        /// - `ssh://user@example.com:2323:/opt/repo`
        name: String,
        /// Obtain the first N archives
        first_n_archives: Option<NonZeroU16>,
        /// Obtain the last N archives
        last_n_archives: Option<NonZeroU16>,
        /// only consider archive names matching the glob.
        glob_archives: Option<String>,
    },
    /// Mount an archive (repo_name::archive_name)
    Archive {
        /// Path to the borg archive you wish to mount
        ///
        /// Example values:
        /// - `/tmp/foo::my-archive`
        /// - `user@example.com:/opt/repo::archive`
        /// - `ssh://user@example.com:2323:/opt/repo`
        archive_name: String,
    },
}

/// Options for [crate::sync::mount]
///
/// Mount an archive or repository as a FUSE filesystem. This is useful for
/// browsing archives or repositories and interactive restoration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MountOptions {
    /// The archive or repo you wish to mount
    pub mount_source: MountSource,
    /// The path where the repo or archive will be mounted.
    ///
    /// Example values:
    /// - `/mnt/my-borg-backup`
    pub mountpoint: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
    /// Paths to extract. If left empty all paths will be present. Useful
    /// for whitelisting certain paths in the backup.
    ///
    /// Example values:
    /// - `/a/path/I/actually/care/about`
    /// - `**/some/intermediate/folder/*`
    pub select_paths: Vec<Pattern>,
}

impl MountOptions {
    /// Create an new [MountOptions]
    pub fn new(mount_source: MountSource, mountpoint: String) -> Self {
        Self {
            mount_source,
            mountpoint,
            passphrase: None,
            select_paths: vec![],
        }
    }
}

/// Options for [crate::sync::compact]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompactOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
}

/// The options for a borg create command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// Name of archive to create (must be also a valid directory name)
    ///
    /// The archive name needs to be unique.
    /// It must not end in ‘.checkpoint’ or ‘.checkpoint.N’ (with N being a number),
    /// because these names are used for checkpoints and treated in special ways.
    ///
    /// In the archive name, you may use the following placeholders:
    /// {now}, {utcnow}, {fqdn}, {hostname}, {user} and some others.
    ///
    /// See <https://borgbackup.readthedocs.io/en/stable/usage/help.html#borg-patterns> for further
    /// information.
    pub archive: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
    /// Add a comment text to the archive
    pub comment: Option<String>,
    /// Specify the compression mode that should be used.
    ///
    /// Defaults to [CompressionMode::Lz4].
    pub compression: Option<CompressionMode>,
    /// The paths to archive.
    ///
    /// All given paths will be recursively traversed.
    pub paths: Vec<String>,
    /// Exclude directories that contain a CACHEDIR.TAG file
    /// (<http://www.bford.info/cachedir/spec.html>)
    pub exclude_caches: bool,
    /// The patterns to apply
    ///
    /// Using these, you may specify the backup roots (starting points)
    /// and patterns for inclusion/exclusion.
    ///
    /// See [PatternInstruction] for further information.
    ///
    /// Note that the order of the instructions is important:
    ///
    /// ```bash
    /// # backup pics, but not the ones from 2018, except the good ones:
    /// borg create --pattern=+pics/2018/good --pattern=-pics/2018 repo::arch pics
    /// ```
    pub patterns: Vec<PatternInstruction>,
    /// read include/exclude patterns from the given path, one per line
    ///
    /// Refer to <https://borgbackup.readthedocs.io/en/stable/usage/help.html#borg-help-patterns>
    /// for further information how to use pattern / exclude files.
    pub pattern_file: Option<String>,
    /// Exclude paths matching PATTERN.
    ///
    /// See [Pattern] for further information.
    pub excludes: Vec<Pattern>,
    /// read exclude patterns from the given path, one per line
    ///
    /// Refer to <https://borgbackup.readthedocs.io/en/stable/usage/help.html#borg-help-patterns>
    /// for further information how to use pattern / exclude files.
    pub exclude_file: Option<String>,
    /// Only store numeric user and group identifiers
    pub numeric_ids: bool,
    /// Detect sparse holes in input (supported only by fixed chunker)
    pub sparse: bool,
    /// Open and read block and char device files as well as FIFOs as if they were regular files.
    ///
    /// Also follows symlinks pointing to these kinds of files.
    pub read_special: bool,
    /// Do not read and store xattrs into archive
    pub no_xattrs: bool,
    /// Do not read and store ACLs into archive
    pub no_acls: bool,
    /// Do not read and store flags (e.g. NODUMP, IMMUTABLE) into archive
    pub no_flags: bool,
}

impl CreateOptions {
    /// Create an new [CreateOptions]
    pub fn new(
        repository: String,
        archive: String,
        paths: Vec<String>,
        patterns: Vec<PatternInstruction>,
    ) -> Self {
        Self {
            repository,
            archive,
            passphrase: None,
            comment: None,
            compression: None,
            paths,
            exclude_caches: false,
            patterns,
            pattern_file: None,
            excludes: vec![],
            exclude_file: None,
            numeric_ids: false,
            sparse: false,
            read_special: false,
            no_xattrs: false,
            no_acls: false,
            no_flags: false,
        }
    }
}

/// The options to provide to the [crate::sync::init] command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// The mode to use for encryption
    pub encryption_mode: EncryptionMode,
    /// Set the repository to append_only mode.
    /// Note that this only affects the low level structure of the repository,
    /// and running delete or prune will still be allowed.
    ///
    /// See [Append-only mode (forbid compaction)](https://borgbackup.readthedocs.io/en/stable/usage/notes.html#append-only-mode)
    /// in Additional Notes for more details.
    pub append_only: bool,
    /// Create the parent directories of the repo, if they are missing.
    ///
    /// Defaults to false
    pub make_parent_dirs: bool,
    /// Set storage quota of the new repository (e.g. 5G, 1.5T)
    ///
    /// No quota is set by default
    pub storage_quota: Option<String>,
}

impl InitOptions {
    /// Create new [InitOptions].
    ///
    /// `append_only`, `make_parent_dirs` and `storage_quota` are set to their defaults.
    pub fn new(repository: String, encryption_mode: EncryptionMode) -> Self {
        Self {
            repository,
            encryption_mode,
            append_only: false,
            make_parent_dirs: false,
            storage_quota: None,
        }
    }
}

/// The options for the [crate::sync::list] command
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListOptions {
    /// Path to the repository
    ///
    /// Example values:
    /// - `/tmp/foo`
    /// - `user@example.com:/opt/repo`
    /// - `ssh://user@example.com:2323:/opt/repo`
    pub repository: String,
    /// The passphrase for the repository
    ///
    /// If using a repository with [EncryptionMode::None],
    /// you can leave this option empty
    pub passphrase: Option<String>,
}

pub(crate) fn init_fmt_args(options: &InitOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options}init -e {e}{append_only}{make_parent_dirs}{storage_quota} {repository}",
        e = options.encryption_mode,
        append_only = if options.append_only {
            " --append-only"
        } else {
            ""
        },
        make_parent_dirs = if options.make_parent_dirs {
            " --make-parent-dirs"
        } else {
            ""
        },
        storage_quota = options
            .storage_quota
            .as_ref()
            .map_or("".to_string(), |x| format!(
                " --storage-quota {quota}",
                quota = shlex::quote(x)
            )),
        repository = shlex::quote(&options.repository),
    )
}

fn log_message(level_name: LevelName, time: f64, name: String, message: String) {
    match level_name {
        LevelName::Debug => debug!("{time} {name}: {message}"),
        LevelName::Info => info!("{time} {name}: {message}"),
        LevelName::Warning => warn!("{time} {name}: {message}"),
        LevelName::Error => error!("{time} {name}: {message}"),
        LevelName::Critical => error!("{time} {name}: {message}"),
    }
}

pub(crate) fn init_parse_result(res: Output) -> Result<(), InitError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(InitError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in res.stderr.lines() {
        let line = line.map_err(InitError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                match msg_id {
                    MessageId::RepositoryAlreadyExists => {
                        return Err(InitError::RepositoryAlreadyExists)
                    }
                    _ => {
                        if exit_code > 1 {
                            return Err(InitError::UnexpectedMessageId(msg_id));
                        }
                    }
                }
            }

            if let Some(MessageId::RepositoryAlreadyExists) = msg_id {
                return Err(InitError::RepositoryAlreadyExists);
            } else if let Some(msg_id) = msg_id {
                return Err(InitError::UnexpectedMessageId(msg_id));
            }
        }
    }

    if exit_code > 1 {
        return Err(InitError::Unknown(output));
    }

    Ok(())
}

pub(crate) fn prune_fmt_args(options: &PruneOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options} prune{keep_within}{keep_secondly}{keep_minutely}{keep_hourly}{keep_daily}{keep_weekly}{keep_monthly}{keep_yearly} {repository}",
        keep_within = options.keep_within.as_ref().map_or("".to_string(), |x| format!(" --keep-within {x}")),
        keep_secondly = options.keep_secondly.as_ref().map_or("".to_string(), |x| format!(" --keep-secondly {x}")),
        keep_minutely = options.keep_minutely.map_or("".to_string(), |x| format!(" --keep-minutely {x}")),
        keep_hourly = options.keep_hourly.map_or("".to_string(), |x| format!(" --keep-hourly {x}")),
        keep_daily = options.keep_daily.map_or("".to_string(), |x| format!(" --keep-daily {x}")),
        keep_weekly = options.keep_weekly.map_or("".to_string(), |x| format!(" --keep-weekly {x}")),
        keep_monthly = options.keep_monthly.map_or("".to_string(), |x| format!(" --keep-monthly {x}")),
        keep_yearly = options.keep_yearly.map_or("".to_string(), |x| format!(" --keep-yearly {x}")),
        repository = shlex::quote(&options.repository)
    )
}

pub(crate) fn prune_parse_output(res: Output) -> Result<(), PruneError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(PruneError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(PruneError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                if exit_code > 1 {
                    return Err(PruneError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code > 1 {
        return Err(PruneError::Unknown(output));
    }

    Ok(())
}

pub(crate) fn mount_fmt_args(options: &MountOptions, common_options: &CommonOptions) -> String {
    let mount_source_formatted = match &options.mount_source {
        MountSource::Repository {
            name,
            first_n_archives,
            last_n_archives,
            glob_archives,
        } => {
            format!(
                "{name}{first_n_archives}{last_n_archives}{glob_archives}",
                name = name.clone(),
                first_n_archives = first_n_archives
                    .map(|first_n| format!(" --first {}", first_n))
                    .unwrap_or_default(),
                last_n_archives = last_n_archives
                    .map(|last_n| format!(" --last {}", last_n))
                    .unwrap_or_default(),
                glob_archives = glob_archives
                    .as_ref()
                    .map(|glob| format!(" --glob-archives {}", glob))
                    .unwrap_or_default(),
            )
        }
        MountSource::Archive { archive_name } => archive_name.clone(),
    };
    format!(
        "--log-json {common_options} mount {mount_source} {mountpoint}{select_paths}",
        mount_source = mount_source_formatted,
        mountpoint = options.mountpoint,
        select_paths = options
            .select_paths
            .iter()
            .map(|x| format!(" --pattern={}", shlex::quote(&x.to_string()),))
            .collect::<Vec<String>>()
            .join(" "),
    )
}

pub(crate) fn mount_parse_output(res: Output) -> Result<(), MountError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(MountError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(MountError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                if exit_code > 1 {
                    return Err(MountError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code > 1 {
        return Err(MountError::Unknown(output));
    }
    Ok(())
}

pub(crate) fn list_fmt_args(options: &ListOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options} list --json {repository}",
        repository = shlex::quote(&options.repository)
    )
}

pub(crate) fn list_parse_output(res: Output) -> Result<ListRepository, ListError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(ListError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(ListError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                match msg_id {
                    MessageId::RepositoryDoesNotExist => {
                        return Err(ListError::RepositoryDoesNotExist);
                    }
                    MessageId::PassphraseWrong => {
                        return Err(ListError::PassphraseWrong);
                    }
                    _ => {
                        if exit_code > 1 {
                            return Err(ListError::UnexpectedMessageId(msg_id));
                        }
                    }
                }
            }
        }
    }

    if exit_code > 1 {
        return Err(ListError::Unknown(output));
    }

    trace!("Parsing output");
    let list_repo: ListRepository = serde_json::from_slice(&res.stdout)?;

    Ok(list_repo)
}

pub(crate) fn create_fmt_args(
    options: &CreateOptions,
    common_options: &CommonOptions,
    progress: bool,
) -> String {
    format!(
        "--log-json{p} {common_options}create --json{comment}{compression}{num_ids}{sparse}{read_special}{no_xattr}{no_acls}{no_flags}{ex_caches}{patterns}{excludes}{pattern_file}{exclude_file} {repo}::{archive} {paths}",
        p = if progress { " --progress" } else { "" },
        comment = options.comment.as_ref().map_or("".to_string(), |x| format!(
            " --comment {}",
            shlex::quote(x)
        )),
        compression = options.compression.as_ref().map_or("".to_string(), |x| format!(" --compression {x}")),
        num_ids = if options.numeric_ids { " --numeric-ids" } else { "" },
        sparse = if options.sparse { " --sparse" } else { "" },
        read_special = if options.read_special { " --read-special" } else { "" },
        no_xattr = if options.no_xattrs { " --noxattrs" } else { "" },
        no_acls = if options.no_acls { " --noacls" } else { "" },
        no_flags = if options.no_flags { " --noflags" } else { "" },
        ex_caches = if options.exclude_caches { " --exclude-caches" } else {""},
        patterns = options.patterns.iter().map(|x| format!(
            " --pattern={}",
            shlex::quote(&x.to_string()),
        )).collect::<Vec<String>>().join(" "),
        excludes = options.excludes.iter().map(|x| format!(
            " --exclude={}",
            shlex::quote(&x.to_string()),
        )).collect::<Vec<String>>().join(" "),
        pattern_file = options.pattern_file.as_ref().map_or(
            "".to_string(),
            |x| format!(" --patterns-from {}", shlex::quote(x)),
        ),
        exclude_file = options.exclude_file.as_ref().map_or(
            "".to_string(),
            |x| format!(" --exclude-from {}", shlex::quote(x)),
        ),
        repo = shlex::quote(&options.repository),
        archive = shlex::quote(&options.archive),
        paths = options.paths.join(" "),
    )
}

pub(crate) fn create_parse_output(res: Output) -> Result<Create, CreateError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(CreateError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(CreateError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                match msg_id {
                    MessageId::ArchiveAlreadyExists => {
                        return Err(CreateError::ArchiveAlreadyExists)
                    }
                    MessageId::PassphraseWrong => {
                        return Err(CreateError::PassphraseWrong);
                    }
                    _ => {
                        if exit_code > 1 {
                            return Err(CreateError::UnexpectedMessageId(msg_id));
                        }
                    }
                }
            }
        }
    }

    if exit_code > 1 {
        return Err(CreateError::Unknown(output));
    }

    trace!("Parsing stats");
    let stats: Create = serde_json::from_slice(&res.stdout)?;

    Ok(stats)
}

pub(crate) fn compact_fmt_args(options: &CompactOptions, common_options: &CommonOptions) -> String {
    format!(
        "--log-json {common_options}compact {repository}",
        repository = shlex::quote(&options.repository)
    )
}

pub(crate) fn compact_parse_output(res: Output) -> Result<(), CompactError> {
    let Some(exit_code) = res.status.code() else {
        warn!("borg process was terminated by signal");
        return Err(CompactError::TerminatedBySignal);
    };

    let mut output = String::new();

    for line in BufRead::lines(res.stderr.as_slice()) {
        let line = line.map_err(CompactError::InvalidBorgOutput)?;
        writeln!(output, "{line}").unwrap();

        trace!("borg output: {line}");

        let log_msg: LoggingMessage = serde_json::from_str(&line)?;

        if let LoggingMessage::LogMessage {
            name,
            message,
            level_name,
            time,
            msg_id,
        } = log_msg
        {
            log_message(level_name, time, name, message);

            if let Some(msg_id) = msg_id {
                if exit_code > 1 {
                    return Err(CompactError::UnexpectedMessageId(msg_id));
                }
            }
        }
    }

    if exit_code > 1 {
        return Err(CompactError::Unknown(output));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use crate::common::{
        mount_fmt_args, prune_fmt_args, CommonOptions, MountOptions, MountSource, Pattern,
        PruneOptions,
    };
    #[test]
    fn test_prune_fmt_args() {
        let mut prune_option = PruneOptions::new("prune_option_repo".to_string());
        prune_option.keep_secondly = NonZeroU16::new(1);
        prune_option.keep_minutely = NonZeroU16::new(2);
        prune_option.keep_hourly = NonZeroU16::new(3);
        prune_option.keep_daily = NonZeroU16::new(4);
        prune_option.keep_weekly = NonZeroU16::new(5);
        prune_option.keep_monthly = NonZeroU16::new(6);
        prune_option.keep_yearly = NonZeroU16::new(7);
        let args = prune_fmt_args(&prune_option, &CommonOptions::default());
        assert_eq!("--log-json  prune --keep-secondly 1 --keep-minutely 2 --keep-hourly 3 --keep-daily 4 --keep-weekly 5 --keep-monthly 6 --keep-yearly 7 prune_option_repo", args);
    }
    #[test]
    fn test_mount_fmt_args() {
        let mount_option = MountOptions::new(
            MountSource::Archive {
                archive_name: "/tmp/borg-repo::archive".to_string(),
            },
            String::from("/mnt/borg-mount"),
        );
        let args = mount_fmt_args(&mount_option, &CommonOptions::default());
        assert_eq!(
            "--log-json  mount /tmp/borg-repo::archive /mnt/borg-mount",
            args
        );
    }
    #[test]
    fn test_mount_fmt_args_patterns() {
        let mut mount_option = MountOptions::new(
            MountSource::Archive {
                archive_name: "/my-borg-repo".to_string(),
            },
            String::from("/borg-mount"),
        );
        mount_option.select_paths = vec![
            Pattern::Shell("**/test/*".to_string()),
            Pattern::Regex("^[A-Z]{3}".to_string()),
        ];
        let args = mount_fmt_args(&mount_option, &CommonOptions::default());
        assert_eq!(
            "--log-json  mount /my-borg-repo /borg-mount --pattern=\"sh:**/test/*\"  --pattern=\"re:^[A-Z]{3}\"",
            args
        );
    }
    #[test]
    fn test_mount_fmt_args_repo() {
        let mut mount_option = MountOptions::new(
            MountSource::Repository {
                name: "/my-repo".to_string(),
                first_n_archives: Some(NonZeroU16::new(10).unwrap()),
                last_n_archives: Some(NonZeroU16::new(5).unwrap()),
                glob_archives: Some("archive-name*12-2022*".to_string()),
            },
            String::from("/borg-mount"),
        );
        mount_option.select_paths = vec![Pattern::Shell("**/foobar/*".to_string())];
        let args = mount_fmt_args(&mount_option, &CommonOptions::default());
        assert_eq!(
            "--log-json  mount /my-repo --first 10 --last 5 --glob-archives archive-name*12-2022* /borg-mount --pattern=\"sh:**/foobar/*\"",
            args
        );
    }
}
