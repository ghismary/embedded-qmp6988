use embedded_qmp6988::{IirFilter, OverSamplingSetting, Qmp6988, DEFAULT_I2C_ADDRESS};
use linux_embedded_hal as hal;

fn main() -> Result<(), embedded_qmp6988::Error<hal::I2CError>> {
    // Create the I2C device from the chosen embedded-hal implementation,
    // in this case linux-embedded-hal
    let i2c = match hal::I2cdev::new("/dev/i2c-1") {
        Err(err) => {
            eprintln!("Could not create I2C device: {}", err);
            std::process::exit(1);
        }
        Ok(i2c) => i2c,
    };

    // Create the sensor and configure its repeatability
    let mut sensor = Qmp6988::new(i2c, DEFAULT_I2C_ADDRESS, hal::Delay {})?;
    sensor.set_filter(IirFilter::Off)?;
    sensor.set_oversampling_setting(OverSamplingSetting::HighSpeed)?;

    // Perform a barometric pressure measurement
    let measurement = sensor.measure()?;
    println!("Pressure: {:.2} hPa", measurement.pressure);
    Ok(())
}
