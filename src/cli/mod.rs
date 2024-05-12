mod args;

use std::path::Path;

use clap::Parser;

use crate::{
    config,
    link::{self, CreateLinkOptions},
    lock::LockFile,
    log::Logger,
    schema,
};

use self::args::{SymlonkArgs, SymlonkCommand, SymlonkCreateSubcommand};

pub fn run() {
    let args = SymlonkArgs::parse();

    match args.command {
        SymlonkCommand::Create(SymlonkCreateSubcommand::Link {
            symlink_name,
            symlink_target,
        }) => {
            // println!("Hello, {:?} -> {:?}", symlink_name, symlink_target);

            let mut create_link_opts = CreateLinkOptions::new(false, false, false);
            link::create_link(
                symlink_name.as_path(),
                symlink_target.as_path(),
                &mut create_link_opts,
            )
            .expect("create symlink");
        }

        SymlonkCommand::Create(SymlonkCreateSubcommand::Links {
            symlink_declarations,
            lock_file: lock_file_path,
            prune,
            verify,
        }) => {
            let log = Logger::default();
            let symlinks = config::parse_symlinks_from_config_files(&symlink_declarations)
                .unwrap_or_else(|error| {
                    log.error(format_args!(
                        "[parse_symlinks_from_config_files] {:?}",
                        error
                    ));
                    panic!()
                });
            let mut lock_file = crate::lock::parse_lock_file(lock_file_path.as_path())
                .unwrap_or_else(|error| {
                    log.debug(format_args!(
                        "lock file {} could not be parsed: {:?}",
                        lock_file_path.to_string_lossy(),
                        error
                    ));
                    match error {
                        crate::lock::ParseLockFileError::Deserialize(deserialize_error) => {
                            log.error(format_args!(
                                "invalid lock file ({}): {:?}",
                                lock_file_path.to_string_lossy(),
                                deserialize_error
                            ));

                            // TODO: graceful exit, don't panic
                            panic!();
                        }
                        crate::lock::ParseLockFileError::Io(io_error) => {
                            match io_error.kind() {
                                std::io::ErrorKind::NotFound => {
                                    let input_create_new_lock_file = log.prompt_char(
                                        format_args!(
                                            "lock file not found: {}. Create a new lock file? [y/N]",
                                            lock_file_path.to_string_lossy(),
                                        )
                                    ).expect("prompt_char").is_some_and(|ch| ch.to_ascii_lowercase() == 'y');

                                    if input_create_new_lock_file {
                                        log.success(format_args!(
                                            "create lock file {}",
                                            lock_file_path.to_string_lossy()
                                        ));
                                        LockFile::new(lock_file_path.clone())
                                    } else {
                                        log.error(format_args!("no lock file, aborting"));
                                        // TODO: graceful exit, don't panic
                                        panic!();
                                    }
                                }
                                _ => {
                                    log.error(format_args!(
                                        "IO error parsing lock file: {:?}",
                                        io_error
                                    ));
                                    // TODO: graceful exit, don't panic
                                    panic!();
                                }
                            }
                        }
                    }
                });

            log.debug(format_args!(
                "Parsed symlinks from all config files: {:#?}",
                symlinks
            ));
            log.debug(format_args!(
                "lock file ({}) {:?}",
                lock_file_path.to_string_lossy(),
                lock_file
            ));

            if prune {
                let symlinks_to_delete = lock_file.get_symlinks_to_delete(&symlinks);

                if symlinks_to_delete.is_empty() {
                    log.info(format_args!("prune: lock file has no outdated symlinks"))
                }
                for (name, _target) in symlinks_to_delete {
                    match std::fs::remove_file(name.as_path()) {
                        Ok(()) => {
                            lock_file.remove_symlink(name.as_path());
                            log.success(format_args!("prune: unlink {}", name.to_string_lossy()))
                        }
                        Err(error) => log.error(format_args!("prune: {}", error)),
                    }
                }
            }

            let mut create_link_options = link::CreateLinkOptions::new(false, false, false);
            for (name, target) in symlinks {
                let created =
                    link::create_link(name.as_path(), target.as_path(), &mut create_link_options)
                        .expect("create_link");
                // if let Some(CreatedSymlink { name, target }) = created {
                if created {
                    let old_target = lock_file.set_symlink(name.as_path(), target.as_path());
                    log.debug(format_args!(
                        "added symlink to lock file: {} -> {}{}",
                        name.to_string_lossy(),
                        target.to_string_lossy(),
                        // old_target.map(|target| target.to_string_lossy().to_string()).unwrap(),
                        // old_target.map_or("".into(), |target| target.to_string_lossy().to_string()),
                        old_target.map_or("".into(), |target| format!(
                            " (was {})",
                            target.to_string_lossy()
                        )),
                    ))
                }
            }

            let serialized_lock_file = lock_file.to_string().expect("toml");

            std::fs::write(lock_file_path.as_path(), serialized_lock_file).expect("write");

            if verify {
                crate::lock::verify(lock_file_path.as_path(), Some(symlink_declarations));
            }
        }

        SymlonkCommand::Create(SymlonkCreateSubcommand::Schema) => {
            schema::to_writer(std::io::stdout()).expect("to_writer");
        }

        SymlonkCommand::Verify {
            lock_file: lock_file_path,
            config_files,
        } => {
            crate::lock::verify(lock_file_path.as_path(), config_files);
        }

        SymlonkCommand::Unlink {
            lock_file: lock_file_path,
        } => {
            let log = Logger::default();
            let mut lock_file =
                crate::lock::parse_lock_file(lock_file_path.as_path()).unwrap_or_else(|error| {
                    log.debug(format_args!(
                        "lock file {} could not be parsed: {:?}",
                        lock_file_path.to_string_lossy(),
                        error
                    ));
                    match error {
                        crate::lock::ParseLockFileError::Deserialize(deserialize_error) => {
                            log.error(format_args!(
                                "invalid lock file ({}): {:?}",
                                lock_file_path.to_string_lossy(),
                                deserialize_error
                            ));

                            // TODO: graceful exit, don't panic
                            panic!();
                        }
                        crate::lock::ParseLockFileError::Io(io_error) => {
                            match io_error.kind() {
                                std::io::ErrorKind::NotFound => {
                                    log.error(format_args!(
                                        "lock file not found: {}",
                                        lock_file_path.to_string_lossy()
                                    ));
                                    panic!();
                                }
                                _ => {
                                    log.error(format_args!(
                                        "IO error parsing lock file: {:?}",
                                        io_error
                                    ));
                                    // TODO: graceful exit, don't panic
                                    panic!();
                                }
                            }
                        }
                    }
                });

            for symlink_name in lock_file.symlinks().keys() {
                let log = Logger::default();
                match std::fs::remove_file(symlink_name) {
                    Ok(()) => {
                        log.success(format_args!("unlink {}", symlink_name.to_string_lossy()))
                    }
                    Err(error) => log.error(format_args!("remove_file: {}", error)),
                }
            }

            log.success(format_args!("deleted {} symlinks", lock_file.symlink_count()));

            lock_file.remove_symlinks();
            let serialized = lock_file.to_string().expect("serialize");
            std::fs::write(lock_file_path.as_path(), serialized).expect("write");
        }
    }
}

// fn unlink(symlink_name: &Path, lock_file: &mut LockFile) {
//     let log = Logger::default();
//     match std::fs::remove_file(symlink_name) {
//         Ok(()) => {
//             lock_file.remove_symlink(symlink_name);
//             log.success(format_args!("unlink {}", symlink_name.to_string_lossy()))
//         }
//         Err(error) => log.error(format_args!("prune: {}", error)),
//     }
// }
