#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs)]
#![no_std]

use embedded_hal::i2c::{Operation, SevenBitAddress};
#[allow(unused_imports)]
use micromath::F32Ext;

/// The I2C address when the SDO pin is connected to logic low
pub const I2C_ADDRESS_LOGIC_LOW: SevenBitAddress = 0x70;
/// The I2C address when the SDO pin is connected to logic high
pub const I2C_ADDRESS_LOGIC_HIGH: SevenBitAddress = 0x56;
/// The default I2C address (SDO pin connected to low)
pub const DEFAULT_I2C_ADDRESS: SevenBitAddress = I2C_ADDRESS_LOGIC_LOW;

const CHIP_ID_REGISTER: u8 = 0xd1;
const COE_B00_1_REGISTER: u8 = 0xa0;
const CTRL_MEAS_REGISTER: u8 = 0xf4;
const IIR_CNT_REGISTER: u8 = 0xf1;
const PRESS_TXD2: u8 = 0xf7;
const RESET_REGISTER: u8 = 0xe0;

/// All possible errors generated when using the Qmp6988 struct
#[derive(Debug)]
pub enum Error<I2cE>
where
    I2cE: embedded_hal::i2c::Error,
{
    /// I²C bus error
    I2c(I2cE),
    /// The QMP6988 chip has not been detected
    ChipNotDetected,
    /// The computed CRC and the one sent by the device mismatch
    BadCrc,
}

impl<I2cE> From<I2cE> for Error<I2cE>
where
    I2cE: embedded_hal::i2c::Error,
{
    fn from(value: I2cE) -> Self {
        Error::I2c(value)
    }
}

/// IIR (Infinite Impulse Response) filter.
///
/// It chooses the amount of noise reduction being performed on the pressure
/// measurement. The greater the coeff, the higher the noise reduction.
#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum IirFilter {
    /// No filter is applied
    Off = 0x00,
    /// A coefficient 2 filter is applied
    Coeff2 = 0x01,
    /// A coefficient 4 filter is applied
    #[default]
    Coeff4 = 0x02,
    /// A coefficient 8 filter is applied
    Coeff8 = 0x03,
    /// A coefficient 16 filter is applied
    Coeff16 = 0x04,
    /// A coefficient 32 filter is applied
    Coeff32 = 0x05,
}

/// The oversampling setting.
///
/// It chooses the accuracy of the measurement, with an impact on the
/// duration of the measurement. The greater the accuracy, the longer the
/// duration of the measurement, and the higher the current consumption.
#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
pub enum OverSamplingSetting {
    /// The shorter measurement, with the lowest accuracy. This is typically
    /// used for weather monitoring.
    HighSpeed,
    /// A measurement with a litle more accuracy, but still a low current
    /// consumption. This might be used for drop detection.
    LowPower,
    /// The standard setting, providing a compromise between the accuracy
    /// of the measurement and its duration. This might be used for elevator
    /// detection.
    #[default]
    Standard,
    /// A high accuracy measurement, with a quite long duration. This might be
    /// used for stair detection.
    HighAccuracy,
    /// The best accuracy measurement, with the longer duration and higher
    /// current consumption. This is typically used for indoor navigation.
    UltraHighAccuracy,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
enum PowerMode {
    #[default]
    Sleep = 0x00,
    Forced = 0x01,
    #[allow(dead_code)]
    Normal = 0x03,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(u8)]
enum OverSampling {
    // Skipped = 0x00,
    #[default]
    X1 = 0x01,
    X2 = 0x02,
    X4 = 0x03,
    X8 = 0x04,
    X16 = 0x05,
    X32 = 0x06,
    // X64 = 0x07,
}

#[derive(Debug, Default)]
struct Coe {
    a0: i32,
    a1: i16,
    a2: i16,
    b00: i32,
    bt1: i16,
    bt2: i16,
    bp1: i16,
    b11: i16,
    bp2: i16,
    b12: i16,
    b21: i16,
    bp3: i16,
}

impl From<&[u8; 25]> for Coe {
    fn from(value: &[u8; 25]) -> Self {
        Coe {
            a0: ((((value[18] as u32) << 12 | (value[19] as u32) << 4 | (value[24] as u32) & 0x0f)
                << 12) as i32)
                >> 12,
            a1: ((value[20] as u16) << 8 | (value[21] as u16)) as i16,
            a2: ((value[22] as u16) << 8 | (value[23] as u16)) as i16,
            b00: ((((value[0] as u32) << 12
                | (value[1] as u32) << 4
                | ((value[24] as u32) & 0xf0) >> 4)
                << 12) as i32)
                >> 12,
            bt1: ((value[2] as u16) << 8 | (value[3] as u16)) as i16,
            bt2: ((value[4] as u16) << 8 | (value[5] as u16)) as i16,
            bp1: ((value[6] as u16) << 8 | (value[7] as u16)) as i16,
            b11: ((value[8] as u16) << 8 | (value[9] as u16)) as i16,
            bp2: ((value[10] as u16) << 8 | (value[11] as u16)) as i16,
            b12: ((value[12] as u16) << 8 | (value[13] as u16)) as i16,
            b21: ((value[14] as u16) << 8 | (value[15] as u16)) as i16,
            bp3: ((value[16] as u16) << 8 | (value[17] as u16)) as i16,
        }
    }
}

#[derive(Debug, Default)]
struct K {
    a0: f32,
    a1: f32,
    a2: f32,
    b00: f32,
    bt1: f32,
    bt2: f32,
    bp1: f32,
    b11: f32,
    bp2: f32,
    b12: f32,
    b21: f32,
    bp3: f32,
}

impl From<&Coe> for K {
    fn from(value: &Coe) -> Self {
        K {
            a0: value.a0 as f32 / 16.0,
            a1: -6.30E-03 + ((4.30E-04 * value.a1 as f32) / 32_767.0),
            a2: -1.90E-11 + ((1.20E-10 * value.a2 as f32) / 32_767.0),
            b00: value.b00 as f32 / 16.0,
            bt1: 1.00E-01 + ((9.10E-02 * value.bt1 as f32) / 32_767.0),
            bt2: 1.20E-08 + ((1.20E-06 * value.bt2 as f32) / 32_767.0),
            bp1: 3.30E-02 + ((1.90E-02 * value.bp1 as f32) / 32_767.0),
            b11: 2.10E-07 + ((1.40E-07 * value.b11 as f32) / 32_767.0),
            bp2: -6.30E-10 + ((3.50E-10 * value.bp2 as f32) / 32_767.0),
            b12: 2.90E-13 + ((7.60E-13 * value.b12 as f32) / 32_767.0),
            b21: 2.10E-15 + ((1.20E-14 * value.b21 as f32) / 32_767.0),
            bp3: 1.30E-16 + ((7.90E-17 * value.bp3 as f32) / 32_767.0),
        }
    }
}

/// The result of a measurement.
///
/// Such a measurement can be obtained using [`Qmp6988::measure()`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Measurement {
    /// The measured barometric pressure (in hPa).
    pub pressure: f32,
    /// The measured temperature (in °C).
    pub temperature: f32,
}

/// QMP6988 device driver
#[derive(Debug)]
pub struct Qmp6988<I2C, D> {
    address: SevenBitAddress,
    coe: Coe,
    delay: D,
    filter: IirFilter,
    i2c: I2C,
    k: K,
    oversampling_setting: OverSamplingSetting,
}

impl<I2C, D> Qmp6988<I2C, D>
where
    I2C: embedded_hal::i2c::I2c,
    D: embedded_hal::delay::DelayNs,
{
    /// Perform a measurement of pressure and temperature.
    ///
    /// This uses the forced power mode to perform a single measurement and
    /// automatically go back to the sleep power mode where the sensor has the
    /// lowest current consumption.
    pub fn measure(&mut self) -> Result<Measurement, Error<I2C::Error>> {
        self.apply_power_mode(PowerMode::Forced)?;
        self.delay.delay_ms(self.get_measurement_duration());
        let mut data = [0u8; 6];
        let mut operations = [Operation::Write(&[PRESS_TXD2]), Operation::Read(&mut data)];
        self.i2c.transaction(self.address, &mut operations)?;
        let dp: &[u8; 3] = &data[0..3].try_into().unwrap();
        let dt: &[u8; 3] = &data[3..6].try_into().unwrap();
        let dp = Self::get_i32_value(dp) - 8_388_608;
        let dt = Self::get_i32_value(dt) - 8_388_608;
        let temperature = self.compensate_temperature(dt);
        let pressure = self.compensate_pressure(dp, temperature);
        Ok(Measurement {
            pressure: pressure / 100.0,
            temperature: temperature / 256.0,
        })
    }

    /// Create a new instance of the QMP6988 device.
    pub fn new(i2c: I2C, address: SevenBitAddress, delay: D) -> Result<Self, Error<I2C::Error>> {
        let mut device = Self {
            address,
            coe: Coe::default(),
            delay,
            filter: IirFilter::default(),
            i2c,
            k: K::default(),
            oversampling_setting: OverSamplingSetting::default(),
        };
        device.check_device()?;
        device.get_calibration_data()?;
        device.apply_filter()?;
        device.apply_measure_control_parameters()?;
        Ok(device)
    }

    /// Perform a soft reset.
    pub fn reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.i2c.write(self.address, &[RESET_REGISTER])?;
        self.delay.delay_ms(10);
        Ok(())
    }

    /// Define the IIR (Infinite Impulse Response) filter to use during the
    /// measurements.
    pub fn set_filter(&mut self, filter: IirFilter) -> Result<(), Error<I2C::Error>> {
        self.filter = filter;
        self.apply_filter()
    }

    /// Define the oversampling setting to use during the measurements.
    pub fn set_oversampling_setting(
        &mut self,
        oversampling_setting: OverSamplingSetting,
    ) -> Result<(), Error<I2C::Error>> {
        self.oversampling_setting = oversampling_setting;
        self.apply_measure_control_parameters()
    }

    fn apply_filter(&mut self) -> Result<(), Error<I2C::Error>> {
        let data = [IIR_CNT_REGISTER, self.filter as u8];
        self.i2c.write(self.address, &data)?;
        self.delay.delay_ms(20);
        Ok(())
    }

    fn apply_measure_control_parameters(&mut self) -> Result<(), Error<I2C::Error>> {
        let (pressure_oversampling, temperature_oversampling) = self.get_oversamplings();
        let data = [
            CTRL_MEAS_REGISTER,
            (temperature_oversampling as u8) << 5
                | (pressure_oversampling as u8) << 2
                | (PowerMode::Sleep as u8),
        ];
        self.i2c.write(self.address, &data)?;
        self.delay.delay_ms(20);
        Ok(())
    }

    fn apply_power_mode(&mut self, power_mode: PowerMode) -> Result<(), Error<I2C::Error>> {
        let mut data = [0u8; 1];
        let mut operations = [
            Operation::Write(&[CTRL_MEAS_REGISTER]),
            Operation::Read(&mut data),
        ];
        self.i2c.transaction(self.address, &mut operations)?;
        let data = [CTRL_MEAS_REGISTER, (data[0] & 0xfc) | power_mode as u8];
        self.i2c.write(self.address, &data)?;
        self.delay.delay_ms(20);
        Ok(())
    }

    fn check_device(&mut self) -> Result<(), Error<I2C::Error>> {
        let mut chip_id = [0u8; 1];
        let mut operations = [
            Operation::Write(&[CHIP_ID_REGISTER]),
            Operation::Read(&mut chip_id),
        ];
        self.i2c.transaction(self.address, &mut operations)?;
        if chip_id[0] == 0x5c {
            Ok(())
        } else {
            Err(Error::ChipNotDetected)
        }
    }

    fn compensate_pressure(&self, dp: i32, temperature: f32) -> f32 {
        let dp = dp as f32;
        self.k.b00
            + self.k.bt1 * temperature
            + self.k.bp1 * dp
            + self.k.b11 * temperature * dp
            + self.k.bt2 * temperature.powf(2.0)
            + self.k.bp2 * dp.powf(2.0)
            + self.k.b12 * dp * temperature.powf(2.0)
            + self.k.b21 * dp.powf(2.0) * temperature
            + self.k.bp3 * dp.powf(3.0)
    }

    fn compensate_temperature(&self, dt: i32) -> f32 {
        let dt = dt as f32;
        self.k.a0 + self.k.a1 * dt + self.k.a2 * dt.powf(2.0)
    }

    fn get_calibration_data(&mut self) -> Result<(), Error<I2C::Error>> {
        let mut coe = [0u8; 25];
        let mut operations = [
            Operation::Write(&[COE_B00_1_REGISTER]),
            Operation::Read(&mut coe),
        ];
        self.i2c.transaction(self.address, &mut operations)?;
        self.coe = (&coe).into();
        self.k = (&self.coe).into();
        Ok(())
    }

    fn get_measurement_duration(&self) -> u32 {
        match self.oversampling_setting {
            OverSamplingSetting::HighSpeed => 6,
            OverSamplingSetting::LowPower => 8,
            OverSamplingSetting::Standard => 11,
            OverSamplingSetting::HighAccuracy => 19,
            OverSamplingSetting::UltraHighAccuracy => 34,
        }
    }

    fn get_oversamplings(&self) -> (OverSampling, OverSampling) {
        match self.oversampling_setting {
            OverSamplingSetting::HighSpeed => (OverSampling::X2, OverSampling::X1),
            OverSamplingSetting::LowPower => (OverSampling::X4, OverSampling::X1),
            OverSamplingSetting::Standard => (OverSampling::X8, OverSampling::X1),
            OverSamplingSetting::HighAccuracy => (OverSampling::X16, OverSampling::X2),
            OverSamplingSetting::UltraHighAccuracy => (OverSampling::X32, OverSampling::X4),
        }
    }

    #[inline]
    fn get_i32_value(data: &[u8; 3]) -> i32 {
        ((data[0] as u32) << 16 | (data[1] as u32) << 8 | (data[2] as u32)) as i32
    }
}

/// Calculate the altitude (in m) from a measurement.
pub fn calculate_altitude(measurement: Measurement) -> f32 {
    ((1_013.25 / measurement.pressure).powf(1.0 / 5.257) - 1.0) * (measurement.temperature + 273.15)
        / 0.0065
}

#[cfg(test)]
mod tests {
    use crate::*;
    use embedded_hal_mock::eh1::delay::StdSleep as Delay;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    fn create_device() -> Qmp6988<I2cMock, Delay> {
        let expectations = [
            I2cTransaction::transaction_start(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [CHIP_ID_REGISTER].to_vec()),
            I2cTransaction::read(DEFAULT_I2C_ADDRESS, [0x5c].to_vec()),
            I2cTransaction::transaction_end(DEFAULT_I2C_ADDRESS),
            I2cTransaction::transaction_start(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [COE_B00_1_REGISTER].to_vec()),
            I2cTransaction::read(
                DEFAULT_I2C_ADDRESS,
                [
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
                    0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                ]
                .to_vec(),
            ),
            I2cTransaction::transaction_end(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [IIR_CNT_REGISTER, 0x02].to_vec()),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [CTRL_MEAS_REGISTER, 0x30].to_vec()),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut device = Qmp6988::new(i2c, DEFAULT_I2C_ADDRESS, Delay {}).unwrap();
        device.i2c.done();
        device
    }

    #[test]
    fn calculate_altitude() {
        assert!(
            (crate::calculate_altitude(Measurement {
                pressure: 991.32,
                temperature: 20.55,
            }) - 188.46)
                .abs()
                < 0.01
        );
        assert!(
            (crate::calculate_altitude(Measurement {
                pressure: 1013.25,
                temperature: 17.93,
            }) - 0.0)
                .abs()
                < 0.01
        );
        assert!(
            (crate::calculate_altitude(Measurement {
                pressure: 1013.25,
                temperature: 37.5,
            }) - 0.0)
                .abs()
                < 0.01
        );
        assert!(
            (crate::calculate_altitude(Measurement {
                pressure: 962.81,
                temperature: 19.37,
            }) - 439.25)
                .abs()
                < 0.01
        );
    }

    #[test]
    fn measure() {
        let expectations = [
            I2cTransaction::transaction_start(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [CTRL_MEAS_REGISTER].to_vec()),
            I2cTransaction::read(DEFAULT_I2C_ADDRESS, [0x30].to_vec()),
            I2cTransaction::transaction_end(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [CTRL_MEAS_REGISTER, 0x31].to_vec()),
            I2cTransaction::transaction_start(DEFAULT_I2C_ADDRESS),
            I2cTransaction::write(DEFAULT_I2C_ADDRESS, [PRESS_TXD2].to_vec()),
            I2cTransaction::read(
                DEFAULT_I2C_ADDRESS,
                [0x00, 0x01, 0x02, 0x00, 0x01, 0x02].to_vec(),
            ),
            I2cTransaction::transaction_end(DEFAULT_I2C_ADDRESS),
        ];
        let mut device = create_device();
        device.i2c.update_expectations(&expectations);
        device.measure().unwrap();
        device.i2c.done();
    }

    #[test]
    fn reset() {
        let expectations = [I2cTransaction::write(
            DEFAULT_I2C_ADDRESS,
            [RESET_REGISTER].to_vec(),
        )];
        let mut device = create_device();
        device.i2c.update_expectations(&expectations);
        device.reset().unwrap();
        device.i2c.done();
    }

    #[test]
    fn set_filter() {
        let expectations = [I2cTransaction::write(
            DEFAULT_I2C_ADDRESS,
            [IIR_CNT_REGISTER, 0x05].to_vec(),
        )];
        let mut device = create_device();
        device.i2c.update_expectations(&expectations);
        device.set_filter(IirFilter::Coeff32).unwrap();
        device.i2c.done();
    }

    #[test]
    fn set_oversampling_setting() {
        let expectations = [I2cTransaction::write(
            DEFAULT_I2C_ADDRESS,
            [CTRL_MEAS_REGISTER, 0x28].to_vec(),
        )];
        let mut device = create_device();
        device.i2c.update_expectations(&expectations);
        device
            .set_oversampling_setting(OverSamplingSetting::HighSpeed)
            .unwrap();
        device.i2c.done();
    }
}
