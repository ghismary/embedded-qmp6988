[![crates.io](https://img.shields.io/crates/v/embedded-qmp6988.svg)](https://crates.io/crates/embedded-qmp6988)
[![License](https://img.shields.io/crates/l/embedded-qmp6988.svg)](https://crates.io/crates/embedded-qmp6988)
[![Documentation](https://docs.rs/embedded-qmp6988/badge.svg)](https://docs.rs/embedded-qmp6988)

# embedded-qmp6988

This is a platform agnostic Rust driver the QMP6988 digital barometric pressure
sensor using the [`embedded-hal`] and [`embedded-hal-async`] traits.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
[`embedded-hal-async`]: https://github.com/rust-embedded/embedded-hal

This driver can be used both synchronously or asynchronously. It defaults to the
synchronous implementation, but you can switch to the asynchronous one by using
the `async` feature.

## The device

The QMP6988 sensors is high accuracy and small size barometric pressure sensor
with low current consumption.

It can be addressed through an I²C or an SPI interface. This driver uses the
I²C interface.

### Documentation:

- [Datasheet](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/datasheet/unit/enviii/QMP6988%20Datasheet.pdf)
- [C++ driver example](https://github.com/m5stack/M5Unit-ENV/tree/master/src)

## Features

- [x] Choose the oversampling setting used for measurements.
- [x] Choose the IIR filter used to reduce the noise in the measurements.
- [x] Perform a single measurement (using the force power mode).
- [x] Do a sofware set.
- [ ] Perform repeated measurements using the normal power mode.

## Usage

To use this driver, import what you need from this crate and an `embedded-hal`
implementation, then instantiate the device.

```rust,no_run
#[cfg(target_os = "linux")]
mod linux {
    use embedded_qmp6988::{IirFilter, OverSamplingSetting, Qmp6988, Temperature, DEFAULT_I2C_ADDRESS};
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
            "Pressure: {:.2} hPa, Temperature: {:.2} °C",
            measurement.barometric_pressure,
            measurement.temperature.celsius().value()
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
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
