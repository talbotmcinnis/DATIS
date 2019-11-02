#[macro_use]
extern crate log;

use std::str::FromStr;

use clap::{App, Arg};
use datis_core::station::{Airfield, Position, Station};
use datis_core::tts::VoiceKind;
use datis_core::weather::StaticWeather;
use datis_core::AtisSrsClient;
use dotenv::dotenv;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .unwrap();

    let matches = App::new("dcs-radio-station")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("frequency")
                .short("f")
                .long("freq")
                .default_value("251000000")
                .help("Sets the SRS frequency (in Hz, e.g. 251000000 for 251MHz)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("gcloud_key")
                .required(true)
                .long("gcloud")
                .env("GCLOUD_KEY"),
        )
        .get_matches();

    let freq = matches.value_of("frequency").unwrap();
    let freq = if let Ok(n) = u64::from_str(freq) {
        n
    } else {
        error!("The provided frequency is not a valid number");
        return Ok(());
    };
    // Calling .unwrap() is safe here because "gcloud_key" is required
    let gcloud_key = matches.value_of("gcloud_key").unwrap();

    let station = Station {
        name: String::from("Test Station"),
        atis_freq: freq,
        traffic_freq: None,
        voice: VoiceKind::StandardB,
        airfield: Airfield {
            name: String::from("Test"),
            position: Position::default(),
            runways: vec![String::from("09"), String::from("26")],
        },
        weather: StaticWeather,
    };
    let mut client = AtisSrsClient::new(station, None, gcloud_key.to_string(), 5002);
    client.start()?;

    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    rx.recv().unwrap();
    client.stop();

    Ok(())
}