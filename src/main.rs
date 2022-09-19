use anyhow::{Context, Result};

mod co2_linux_reader;
mod co2_parse;

use co2_parse::Message;

fn main() -> Result<()> {
    let reader = co2_linux_reader::DataReader::new()
        .context("Failed to open CO2 monitor. Is it connected?")?;

    loop {
        let buf = reader
            .read()
            .context("Failed to read data from the CO2 monitor. Was it disconnected?")?;

        match Message::try_from(&buf) {
            Ok(m) => match m {
                Message::Co2Level { ppm } => println!("{} ppm CO2", ppm),
                Message::Temperature { celsius } => println!("{:.1} C", celsius),
            },
            Err(e) => println!("Skipping message: {}", e),
        }
    }
}
