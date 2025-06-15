use std::str::from_utf8;
use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::Parser;
use easy_repl::{command, Repl, CommandStatus};
use log::info;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};

mod config;

mod commands;
use commands::CommandController;

mod persist;
mod wol;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config_file: String,
}

fn main() {
    let args = Cli::parse();
    let config = config::parse_config(PathBuf::from_str(args.config_file.as_str()).unwrap())
        .with_context(|| format!("Failed to parse file {:#?}", args.config_file))
        .unwrap();

    let stdout = ConsoleAppender::builder().build();
    let log_config = Config::builder()
        .logger(Logger::builder().build("rustyline", log::LevelFilter::Off))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(config.log.level))
        .unwrap();

    log4rs::init_config(log_config).expect("Failed to initialize logging configuration");

    info!("Starting database!");
    info!("Data path: {:?}", config.data.path);
    info!("Log level: {:?}", config.log.level);

    let command_controller = Arc::new(
        CommandController::new(config.data.path)
            .with_context(|| "Could not create command controller")
            .unwrap(),
    );

    let mut repl = Repl::builder()
        .add(
            "set",
            {
                let command_controller = Arc::clone(&command_controller);
                command! {
                    "Attributes a value to a key",
                    (key: String, value: String) => |key: String, value: String| {
                        match command_controller.clone().set(key.as_str(), value.as_bytes()) {
                            Ok(_) => println!("{} set to {}", key, value),
                            Err(e) => println!("{:?}", e)
                        }
                        Ok(CommandStatus::Done)
                    }
                }
            },
        )
        .add(
            "delete",
            {
                let command_controller = Arc::clone(&command_controller);
                command! {
                    "Deletes a key",
                    (key: String) => |key: String| {
                        match command_controller.clone().delete(key.as_str()) {
                            Ok(_) => println!("Key {} deleted", key),
                            Err(e) => println!("{:?}", e)
                        }
                        Ok(CommandStatus::Done)
                    }
                }
            },
        )
        .add(
            "get",
            {
                let command_controller = Arc::clone(&command_controller);
                command! {
                    "Gets a value from a key",
                    (key: String) => |key: String| {
                        match command_controller.clone().get(key.as_str()) {
                            Ok(Some(data)) => println!("{}", from_utf8(&data).map_or("Could not convert data to string", |s| s)),
                            Ok(None) => println!("Key {} not found", key),
                            Err(e) => println!("{:?}", e)
                        }
                        Ok(CommandStatus::Done)
                    }
                }
            },
        )
        .build()
        .expect("Failed to create repl");

    repl.run().expect("Critical REPL error");
}
