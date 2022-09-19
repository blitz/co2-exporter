use anyhow::{Context, Result};

mod co2_parse;

use co2_parse::Message;

fn main() -> Result<()> {
    let api = hidapi::HidApi::new().context("Failed to initialize hidapi")?;

    let (vid, pid) = (0x04d9, 0xa052);
    let device = api
        .open(vid, pid)
        .context("Failed to open USB device. Is the CO2 monitor attached?")?;

    device
        .send_feature_report(&[0; 8])
        .context("Failed to send feature report")?;

    loop {
        let mut buf = [0u8; 8];

        let res = device
            .read(&mut buf[..])
            .context("Failed to read data from the CO2 monitor")?;
        if res != buf.len() {
            println!("Invalid length {}", res);
            continue;
        }

        match Message::try_from(&buf) {
            Ok(m) => match m {
                Message::Co2Level { ppm } => println!("{} ppm CO2", ppm),
                Message::Temperature { celsius } => println!("{:.1} C", celsius),
            },
            Err(e) => println!("Skipping message: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn decode_works() {
        let magic_table: [u8; 8] = [0; 8];

        assert_eq!(
            decode([183, 164, 50, 182, 200, 154, 156, 80], magic_table),
            [66, 18, 192, 20, 13, 0, 0, 0]
        );
    }

    #[test]
    fn checksum_works() {
        assert!(has_ok_checksum([66, 18, 192, 20, 13, 0, 0, 0]));
    }
}
