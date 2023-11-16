#![windows_subsystem = "windows"]
mod configuration;
mod home_assistant;
mod teams;
mod traits;
mod tray;
mod utils;

use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time;

use crate::configuration::get_configuration;
use crate::teams::api::TeamsAPI;
use crate::tray::create_tray;
use dotenv::dotenv;
use home_assistant::api::HaApi;
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

    let log_config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(log_config)?;

    info!("--------------------");
    info!("Application starting");
    dotenv().ok();

    run().await;

    info!("Application closing");

    exit(0);
}

async fn run() {
    // used by tray icon to allow exiting the application
    let toggle_mute = Arc::new(AtomicBool::new(false));
    let is_running = Arc::new(AtomicBool::new(true));
    let _tray = create_tray(is_running.clone(), toggle_mute.clone());
    let one_second = time::Duration::from_secs(1);

    while is_running.load(Ordering::Relaxed) {
        let conf = get_configuration();
        let ha_api = Arc::new(HaApi::new(conf.ha));
        let teams_api = TeamsAPI::new(&conf.teams);

        teams_api
            .start_listening(ha_api, is_running.clone(), toggle_mute.clone())
            .await;
        // will be for handling a retry loop, rn it does nothing more than slow the exit
        tokio::time::sleep(one_second).await;
    }
}

// todo: ensure Teams connection can be lost and reconnected since it is WS and not REST
// todo: translations & language config?
// todo: get a better icon
// todo: auto create versions and packages when creating tags on GitHub (if doable)
// todo: write new tests and pass existing ones
// todo: improve utils.rs encryption
