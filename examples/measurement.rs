#[cfg(target_os = "linux")]
mod linux {
    use embedded_qmp6988::{
        IirFilter, OverSamplingSetting, Qmp6988, Temperature, DEFAULT_I2C_ADDRESS,
    };
    use linux_embedded_hal as hal;

    pub fn main() -> Result<(), embedded_qmp6988::Error<hal::I2CError>> {
        // Create the I2C device from the chosen embedded-hal implementation,
        // in this case linux-embedded-hal
        let mut i2c = match hal::I2cdev::new("/dev/i2c-1") {
            Err(err) => {
                eprintln!("Could not create I2C device: {}", err);
                std::process::exit(1);
            }
            Ok(i2c) => i2c,
        };
        if let Err(err) = i2c.set_slave_address(DEFAULT_I2C_ADDRESS as u16) {
            eprintln!("Could not set I2C slave address: {}", err);
            std::process::exit(1);
        }

        // Create the sensor and configure its repeatability
        let mut sensor = Qmp6988::new(i2c, DEFAULT_I2C_ADDRESS, hal::Delay {})?;
        sensor.set_filter(IirFilter::Off)?;
        sensor.set_oversampling_setting(OverSamplingSetting::HighSpeed)?;

        // Perform a barometric pressure measurement
        let measurement = sensor.measure()?;
        println!(
            "Pressure: {:.2} hPa, Temperature: {:.2} °C, Altitude: {:.2} m",
            measurement.barometric_pressure.value(),
            measurement.temperature.celsius().value(),
            measurement.altitude().value(),
        );
        Ok(())
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    linux::main();
    #[cfg(not(target_os = "linux"))]
    println!("This example only works on Linux");
}
