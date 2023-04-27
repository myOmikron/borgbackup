use log::{debug, info};

use crate::asynchronous::execute_borg;
use crate::common::{mount_fmt_args, mount_parse_output, CommonOptions, MountOptions};
use crate::errors::MountError;

/// Mount an archive or repo as a FUSE filesystem.
///
/// **Parameter**:
/// - `options`: Reference to [MountOptions]
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn mount(
    options: &MountOptions,
    common_options: &CommonOptions,
) -> Result<(), MountError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = mount_fmt_args(options, common_options);
    debug!("Calling borg: {local_path} {args}");
    let args = shlex::split(&args).ok_or(MountError::ShlexError)?;
    let res = execute_borg(local_path, args, &options.passphrase).await?;

    mount_parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}

/// Unmount a previously mounted archive or repository.
///
/// **Parameter**:
/// - `mountpoint`: The mountpoint to be unmounted.
/// - `common_options`: The [CommonOptions] that can be applied to any command
pub async fn umount(mountpoint: String, common_options: &CommonOptions) -> Result<(), MountError> {
    let local_path = common_options.local_path.as_ref().map_or("borg", |x| x);

    let args = vec!["umount".to_string(), mountpoint];
    let res = execute_borg(local_path, args, &None).await?;

    mount_parse_output(res)?;

    info!("Finished pruning");

    Ok(())
}
