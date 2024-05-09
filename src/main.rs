use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod link;
mod log;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct SymlonkArgs {
    #[command(subcommand)]
    command: SymlonkCommand,
}

#[derive(Subcommand, Debug)]
enum SymlonkCommand {
    #[command(subcommand)]
    Create(SymlonkCreateSubcommand),
}

#[derive(Subcommand, Debug)]
enum SymlonkCreateSubcommand {
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

fn main() {
    let args = SymlonkArgs::parse();

    match args.command {
        SymlonkCommand::Create(SymlonkCreateSubcommand::Link {
            symlink_name,
            symlink_target,
        }) => {
            println!("Hello, {:?} -> {:?}", symlink_name, symlink_target);

            let mut create_link_opts = link::CreateLinkOptions::new(false, false, false);
            link::create_link(
                symlink_name.as_path(),
                symlink_target.as_path(),
                &mut create_link_opts,
            )
            .expect("create symlink");
        }

        SymlonkCommand::Create(SymlonkCreateSubcommand::Links { symlink_declarations }) => {
            todo!()
        }
    }
}
