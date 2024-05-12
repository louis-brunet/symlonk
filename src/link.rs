use std::{
    ffi::OsString,
    io,
    os::unix,
    path::Path,
    time::SystemTime,
};

use crate::log;

pub struct CreateLinkOptions {
    overwrite_all: bool,
    backup_all: bool,
    skip_all: bool,
}

impl CreateLinkOptions {
    pub fn new(
        overwrite_all: bool,
        backup_all: bool,
        skip_all: bool,
    ) -> Self {
        Self {
            overwrite_all,
            backup_all,
            skip_all,
        }
    }
}

struct ParseInputError;

enum CreateLinkPromptAction {
    Overwrite,
    Backup,
    Skip,
    OverwriteAll,
    BackupAll,
    SkipAll,
}

impl TryFrom<char> for CreateLinkPromptAction {
    type Error = ParseInputError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'o' => Ok(Self::Overwrite),
            'O' => Ok(Self::OverwriteAll),
            'b' => Ok(Self::Backup),
            'B' => Ok(Self::BackupAll),
            's' => Ok(Self::Skip),
            'S' => Ok(Self::SkipAll),
            _ => Err(ParseInputError),
        }
    }
}

pub fn create_link(
    link_name: &Path,
    link_target: &Path,
    options: &mut CreateLinkOptions,
) -> std::io::Result<bool> {
    // let link_name = options.destination_dir.join(link_name);
    // let link_target = options.source_dir.join(link_target);

    let does_destination_exist = crate::path::path_exists(link_name)?;
    let is_all_action = options.overwrite_all || options.backup_all || options.skip_all;
    let mut action = None;
    let mut overwrite = options.overwrite_all;
    let mut backup = options.backup_all;
    let mut skip = options.skip_all;
    let log = log::Logger::default();

    if does_destination_exist && !is_all_action {
        let current_target = if link_name.is_symlink() {
            link_name.read_link().expect("read_link")
        } else {
            link_name.to_path_buf()
        };

        if current_target.as_path() == link_target {
            log.info(
                format_args!(
                    "skip {}, already linked to {}",
                    link_name.to_string_lossy(),
                    link_target.to_string_lossy()
                ),
            );
            skip = true;
        } else {
            action = prompt_existing_destination(link_name, link_target)?;
        }
    }

    match action {
        Some(CreateLinkPromptAction::Overwrite) => overwrite = true,
        Some(CreateLinkPromptAction::Backup) => backup = true,
        Some(CreateLinkPromptAction::Skip) => skip = true,
        Some(CreateLinkPromptAction::OverwriteAll) => options.overwrite_all = true,
        Some(CreateLinkPromptAction::BackupAll) => options.backup_all = true,
        Some(CreateLinkPromptAction::SkipAll) => options.skip_all = true,
        None => (),
    }

    if skip || options.skip_all {
        // log.info(format!("skipped {}", link_name.to_string_lossy()).as_str())
    } else {
        if backup || options.backup_all {
            let mut backup_name = OsString::from(link_name);
            backup_name.push(".backup");

            if crate::path::path_exists(backup_name.as_os_str())? {
                let timestamp = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                log.info(format_args!("{} already exists", backup_name.to_string_lossy()));
                backup_name = OsString::from(format!(
                    "{}.{}.backup",
                    link_name.to_string_lossy(),
                    timestamp
                ));
            }

            std::fs::rename(link_name, backup_name.as_os_str())?;

            log.success(
                format_args!(
                    "moved {} to {}",
                    link_name.to_string_lossy(),
                    backup_name.to_string_lossy(),
                ),
            )
        }

        if overwrite || options.overwrite_all {
            std::fs::remove_file(link_name).expect("remove_file");
            log.success(format_args!("removed {}", link_name.to_string_lossy()))
        }

        let link_parent = link_name.parent();
        if let Some(parent_path) = link_parent {
            if !parent_path.exists() {
                std::fs::create_dir_all(parent_path)?;
                log.success(
                    format_args!("created directory {}", parent_path.to_string_lossy()),
                );
            } else if !parent_path.is_dir() {
                log.error(
                    format_args!(
                        "symlink parent is not a directory: {}",
                        parent_path.to_string_lossy(),
                    ),
                );
                return Ok(false);
            }
        }

        unix::fs::symlink(link_target, link_name).expect("symlink");
        log.success(
            format_args!(
                "linked {} to {}",
                link_name.to_string_lossy(),
                link_target.to_string_lossy()
            ),
        );

        return Ok(true);
    }

    Ok(false)
}

fn prompt_existing_destination(
    link_name: &Path,
    link_target: &Path,
) -> io::Result<Option<CreateLinkPromptAction>> {
    println!(
        "File already exists: {} (trying to link to {}), what do you want to do?\n[s]kip, [S]kip all, [o]verwrite, [O]verwrite all, [b]ackup, [B]ackup all?",
        link_name.to_string_lossy(),
        link_target.to_string_lossy(),
        // link_target.file_name().unwrap().to_string_lossy(),
    );

    let mut input_buf = String::new();
    std::io::stdin().read_line(&mut input_buf)?;

    let input_char = input_buf.chars().next();
    Ok(input_char.and_then(|ch| CreateLinkPromptAction::try_from(ch).ok()))
}
