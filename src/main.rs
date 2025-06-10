use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, anyhow};
use clap::Parser;
use log::info;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};

mod config;

mod persist;
use persist::write::bwrite1;

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
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(config.log.level))
        .unwrap();

    log4rs::init_config(log_config).expect("Failed to initialize logging configuration");

    info!("Starting database!");
    info!("Data path: {:?}", config.data.path);
    info!("Log level: {:?}", config.log.level);

    bwrite1(
        config.data.path.join("test_file.txt"),
        vec![0, 2, 7, 3],
    )
    .with_context(|| anyhow!("bwrite1 failed"))
    .unwrap()
}
