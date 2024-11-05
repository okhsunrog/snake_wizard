use log::info;
use std::{thread, time::Duration};
use uom::si::{
    f32::{Pressure, Ratio, ThermodynamicTemperature},
    pressure::{hectopascal, millimeter_of_mercury},
    thermodynamic_temperature::degree_celsius,
};

use anyhow::Result;
use bme280_rs::{Bme280, Configuration, Oversampling, SensorMode};
use esp_idf_svc::hal::{delay::Delay, i2c::*, peripherals::Peripherals, prelude::*};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let p = Peripherals::take()?;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(p.i2c0, p.pins.gpio9, p.pins.gpio10, &config)?;

    let bme280_addr = 0x76;
    info!("Setting up BME280");
    let mut bme280 = Bme280::new_with_address(i2c, bme280_addr, Delay::default());
    bme280.init()?;
    bme280.set_sampling_configuration(
        Configuration::default()
            .with_temperature_oversampling(Oversampling::Oversample1)
            .with_pressure_oversampling(Oversampling::Oversample1)
            .with_humidity_oversampling(Oversampling::Oversample1)
            .with_sensor_mode(SensorMode::Normal),
    )?;
    thread::sleep(Duration::from_secs(1));

    loop {
        match bme280.read_sample() {
            Ok(sample) => {
                let temp: ThermodynamicTemperature =
                    sample.temperature.expect("No temperature").into();
                let pressure: Pressure = sample.pressure.expect("No pressure").into();
                let humidity: Ratio = sample.humidity.expect("No humidity");

                // Convert to desired units
                let temp_c = temp.get::<degree_celsius>();
                let pressure_hpa = pressure.get::<hectopascal>();
                // Alternatively for mmHg:
                let pressure_mmhg = pressure.get::<millimeter_of_mercury>();
                let humidity_percent = humidity.value * 100.0;

                println!(
                    "Temperature: {:.1} °C, Pressure: {:.1} hPa ({:.1} mmHg), Humidity: {:.1}%",
                    temp_c, pressure_hpa, pressure_mmhg, humidity_percent
                );
            }
            Err(e) => {
                println!("Failed to read sample: {:?}", e);
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
