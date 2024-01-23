This is a platform agnostic Rust driver the QMP6988 digital barometric pressure
sensor using the [`embedded-hal`] traits.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal

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
- [ ] Include a no floating-point variant for systems without fpu.

## Usage

To use this driver, import what you need from this crate and an `embedded-hal`
implentation, then instatiate the device.

```rust,no_run

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
