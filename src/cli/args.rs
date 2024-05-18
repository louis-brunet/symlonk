use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::lock::LockFile;

/// Symlink management tool that uses a lock file to track create symlinks
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct SymlonkArgs {
    #[command(subcommand)]
    pub command: SymlonkCommand,
}

#[derive(Subcommand, Debug)]
pub enum SymlonkCommand {
    /// Create one or many symlinks
    #[command(subcommand)]
    Create(SymlonkCreateSubcommand),

    /// Verify that the lock file matches config, that all symlinks in the lock file are created, and that symlinks point to existing files
    Verify {
        #[arg(short, long)]
        config_files: Option<Vec<PathBuf>>,

        #[arg(default_value = LockFile::DEFAULT_LOCK_FILE_PATH)]
        lock_file: PathBuf,
    },

    /// Delete all created symlinks stored in the given lock file from the file system
    Unlink {
        #[arg(default_value = LockFile::DEFAULT_LOCK_FILE_PATH)]
        lock_file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum SymlonkCreateSubcommand {
    /// Create one symlink
    Link {
        /// Path of the symlink that will be created
        #[arg()]
        symlink_name: PathBuf,

        /// Path to which the symlink should point
        #[arg()]
        symlink_target: PathBuf,
    },

    /// Create symlinks from symlink declaration files
    Links {
        /// List of paths to symlink declaration files
        #[arg(required = true)]
        symlink_declarations: Vec<PathBuf>,

        /// Path of a symlink declaration file
        #[arg(short, long, default_value = LockFile::DEFAULT_LOCK_FILE_PATH)]
        lock_file: PathBuf,

        /// Delete symlinks that are in lock file but not in config
        #[arg(short, long, default_value_t = false)]
        prune: bool,

        /// Verify that the lock file matches config, that all symlinks in
        /// the lock file are created, and that symlinks point to existing files
        #[arg(short, long, default_value_t = false)]
        verify: bool,

        // TODO: add argument --dry-run ?

        // #[arg(short, long, default_value_t = false)]
        // overwrite: bool,
        //
        // #[arg(short, long, default_value_t = false)]
        // backup: bool,
        //
        // #[arg(short, long, default_value_t = false)]
        // skip: bool,
    },

    /// Generate a JSON schema for symlonk configuration files
    Schema,
    // Example,
}
