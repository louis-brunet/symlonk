use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct SymlonkArgs {
    #[command(subcommand)]
    pub command: SymlonkCommand,
}

#[derive(Subcommand, Debug)]
pub enum SymlonkCommand {
    #[command(subcommand)]
    Create(SymlonkCreateSubcommand),
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

    /// Create symlinks from a symlink declaration file
    Links {
        /// Path of a symlink declaration file
        #[arg()]
        symlink_declarations: PathBuf,
        // /// Path of a symlink declaration file
        // #[arg(short, long)]
        // lock_file: PathBuf,

        // #[arg(short, long, default_value_t = false)]
        // overwrite: bool,
        //
        // #[arg(short, long, default_value_t = false)]
        // backup: bool,
        //
        // #[arg(short, long, default_value_t = false)]
        // skip: bool,
    },
}

