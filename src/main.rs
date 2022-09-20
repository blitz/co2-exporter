#[macro_use]
extern crate rocket;

mod co2_linux_reader;
mod co2_parse;

use anyhow::{Context, Result};
use rocket::{tokio::sync::Mutex, State};
use std::{sync::Arc, thread};

use co2_parse::Message;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct MetricValues {
    co2_ppm: Option<u16>,
    temp_celsius: Option<f32>,
}

#[derive(Debug, Default, Clone)]
struct Metrics {
    values: Arc<Mutex<MetricValues>>,
}

impl Metrics {
    fn update(&self, msg: Message) {
        match msg {
            Message::Co2Level { ppm } => self.values.blocking_lock().co2_ppm = Some(ppm),
            Message::Temperature { celsius } => {
                self.values.blocking_lock().temp_celsius = Some(celsius)
            }
        }
    }
}

#[derive(Responder)]
#[response(
    status = 200,
    content_type = "application/openmetrics-text; version=1.0.0; charset=utf-8"
)]
struct MetricsResponse(String);

#[get("/metrics")]
async fn metrics_endpoint(state: &State<Metrics>) -> Option<MetricsResponse> {
    let metrics = state.values.lock().await;

    Some(MetricsResponse(format!(
        "# TYPE ambient_temperature_celsius gauge
# HELP ambient_temperature_celsius Ambient Temperature in degrees Celsius
ambient_temperature_celsius {:.1}
# TYPE co2_concentration_ppm gauge
# HELP co2_concentration_ppm The CO2 concentration in the air in PPM
co2_concentration_ppm {}
# EOF
",
        // If CO2 or temperature is not there yet, we export no metrics.
        metrics.temp_celsius?,
        metrics.co2_ppm?,
    )))
}

#[rocket::main]
async fn main() -> Result<()> {
    let metrics = Metrics::default();
    let reader = co2_linux_reader::DataReader::new()
        .context("Failed to open CO2 monitor. Is it connected?")?;

    let reader_metrics = metrics.clone();
    let _reader_thread = thread::spawn(move || -> Result<()> {
        loop {
            let buf = reader
                .read()
                .context("Failed to read data from the CO2 monitor. Was it disconnected?")?;

            match Message::try_from(&buf) {
                Ok(m) => reader_metrics.update(m),
                Err(_e) => (),
            }
        }
    });

    let _rocket = rocket::build()
        .manage(metrics)
        .mount("/", routes![metrics_endpoint])
        .ignite()
        .await?
        .launch()
        .await?;

    // TODO We should exit the reader thread here.

    Ok(())
}
