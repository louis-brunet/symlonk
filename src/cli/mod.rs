mod args;

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
                    // FIXME: add prompt "create new symlonk project?", auto creating a lock file
                    // could lead to confusion if user launched from wrong dir or with wrong lock
                    // file path
                    log.info(format_args!(
                        "lock file {} not found, creating",
                        lock_file_path.to_string_lossy()
                    ));
                    LockFile::new(lock_file_path.clone())
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
                            log.success(format_args!("prune: unlinked {}", name.to_string_lossy()))
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

            let serialized_lock_file = toml::to_string(&lock_file).expect("toml");

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
    }
}
