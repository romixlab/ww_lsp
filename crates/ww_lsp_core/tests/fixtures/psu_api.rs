//! A bench power supply's API, a fixture for ww_lsp's hints: exact_units types in properties, streams, sinks and
//! methods of `#[ww_trait]` traits.

use exact_units::origins::Celsius;
use exact_units::prefix::{Centi, Milli, Pow2, Rational};
use exact_units::units::{Ampere, Kelvin, Ohm, Second, Volt, Watt};
use exact_units::{Fixed, Millis, Point, Q15, Range};
use wire_weaver::prelude::*;

/// The heatsink temperature as the sensor reports it.
type Heatsink = Point<Celsius, Fixed<Kelvin, Centi, i16>>;

#[ww_api_root]
pub trait Psu {
    ww_impl!(output: Output);
    ww_impl!(status: Status);
}

#[ww_trait]
pub trait Output {
    /// The voltage setpoint.
    property!(rw v_set: Fixed<Volt, Milli, i16>);
    /// The current limit, 0..5 A.
    property!(rw i_limit: Fixed<Ampere, Milli, u16, Range<0, 5000>>);
    /// The measured output voltage.
    property!(ro+observe+batched v_out: Fixed<Volt, Milli, i16>);
    /// Soft start ramp time.
    property!(rw ramp: Millis<u32>);
    /// Trim, a fraction of full scale.
    property!(rw trim: Q15);
    /// The output's power, an error when the meter is off.
    property!(ro power: Fixed<Watt, Pow2<-8>, u32>, MeterError);
    /// Not an exact_units type: no hint.
    property!(rw label: &'i str);
    /// Every sample of the output current while observed.
    stream!(i_out: Fixed<Ampere, Rational<1, 4096>, i16>);
    /// Sets a load line: output resistance in mΩ, for how long.
    fn set_droop(r: Fixed<Ohm, Milli, u16>, hold: Option<Fixed<Second, Milli, u32>>) -> Result<(), DroopError>;
}

#[ww_trait]
pub trait Status {
    /// The heatsink.
    property!(ro+observe heatsink: Heatsink);
    /// The board temperature, written out.
    property!(ro board: Point<Celsius, Fixed<Kelvin, Centi, i16>>);
}

/// Outside a ww_trait: no hint.
pub trait Plain {
    fn v(&self) -> Fixed<Volt, Milli, i16>;
}
