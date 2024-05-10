use std::path::PathBuf;

use args::{SymlonkArgs, SymlonkCommand, SymlonkCreateSubcommand};
use clap::Parser;

use crate::config::ConfigFile;

mod args;
mod config;
// mod lexer;
mod link;
mod log;
//
// mod parser;

fn main() {
    let args = SymlonkArgs::parse();

    match args.command {
        SymlonkCommand::Create(SymlonkCreateSubcommand::Link {
            symlink_name,
            symlink_target,
        }) => {
            println!("Hello, {:?} -> {:?}", symlink_name, symlink_target);

            let mut create_link_opts = link::CreateLinkOptions::new(
                false,
                false,
                false,
                PathBuf::from("."),
                PathBuf::from("."),
            );
            link::create_link(
                symlink_name.as_path(),
                symlink_target.as_path(),
                &mut create_link_opts,
            )
            .expect("create symlink");
        }

        SymlonkCommand::Create(SymlonkCreateSubcommand::Links {
            symlink_declarations,
        }) => {
            let log = log::Logger::new(None);
            let file_contents =
                std::fs::read_to_string(symlink_declarations.as_path()).expect("read_to_string");
            let parsed_file: ConfigFile = match toml::from_str(file_contents.as_str()) {
                Ok(config) => config,
                Err(error) => {
                    log.error(
                        format!(
                            "Error when parsing file {}: {}",
                            symlink_declarations.to_str().unwrap(),
                            error.message()
                        )
                        .as_str(),
                    );
                    return;
                }
            };

            log.success(format!("parsed file {} : {:?}", symlink_declarations.to_str().unwrap(), parsed_file).as_str());

            let mut options = link::CreateLinkOptions::new(
                false,
                false,
                false,
                parsed_file.config.source_dir,
                parsed_file.config.destination_dir,
            );
            for (name, target) in parsed_file.symlinks {
                link::create_link(name.as_path(), target.as_path(), &mut options)
                    .expect("create_link");
            }
            // todo!()
        }
    }
}
