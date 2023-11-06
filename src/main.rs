// TODO: This actually breaks the app, but it would prevent the command line from opening
// #![windows_subsystem = "windows"]
mod configuration;
mod ha_api;
mod ha_configuration;
mod teams_api;
mod teams_configuration;
mod teams_states;
mod traits;
mod tray_windows;
mod utils;

use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::configuration::{load_configuration, Configuration};
use crate::teams_api::TeamsAPI;
use crate::tray_windows::create_tray;
use dotenv::dotenv;
use ha_api::HaApi;
use log::{info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d:<36} {l} {t} - {m}{n}")))
        .build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    info!("--------------------");
    info!("Application starting");
    dotenv().ok();
    let conf = load_configuration();

    run(conf).await;

    info!("Application closing");

    exit(0);
}

async fn run(conf: Configuration) {
    let ha_api = Arc::new(HaApi::new(&conf.ha));
    let teams_api = TeamsAPI::new(&conf.teams);
    // used by try icon to allow exiting the application
    let is_running = Arc::new(AtomicBool::new(true));
    let _tray = create_tray(is_running.clone());

    teams_api.start_listening(ha_api, is_running).await;
}

// todo: write new tests and pass existing ones
// todo: ensure Teams connection can be lost and reconnected since it is WS and not REST
// todo: logging
// todo: try to trigger an initial status response
// todo: translations & language config
// todo: config
// todo: doc
// todo: auto create versions and packages when creating tags on GitHub (if doable)
// todo: fix the command prompt
// todo: encrypt tokens
