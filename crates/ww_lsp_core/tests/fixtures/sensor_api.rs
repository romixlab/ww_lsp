//! A sensor board's API, a fixture for ww_lsp's semantic tokens: every item macro and access mode, WireWeaver
//! attributes, wrappers, aliases, units, prefixes and origins, and a type of the file's own that shares a unit's
//! name.

use exact_units::origins::{Angle, Celsius};
use exact_units::prefix::{Kibi, Milli, One};
use exact_units::units::{Degree, Kelvin, Pascal, Second, Volt};
use exact_units::{Fixed, Index, Kibibytes, Millis, Point, Q15};
use wire_weaver::prelude::*;
use ww_si::Timed;

/// A row of the file's own: not exact_units' `Row` unit.
#[derive_shrink_wrap]
#[ww_repr(u8)]
pub struct Row {
    #[flag]
    valid: bool,
    pressure: Fixed<Pascal, Kibi, u16>,
}

#[ww_trait]
pub trait Sensor {
    /// The temperature, sampled.
    property!(ro+observe+batched t: Timed<Point<Celsius, Fixed<Kelvin, Milli, i16>>, Millis<u32>>);
    /// Heading, wraps every turn.
    property!(rw heading: Point<Angle<Degree>, Fixed<Degree, One, i16>>);
    /// Calibration gain, write only.
    property!(wo gain: Q15<Volt>);
    /// Rows the sensor logged.
    stream!(rows: Row);
    /// Firmware, a chunk at a time.
    sink!(firmware: Kibibytes<u16>);
    /// Reads one logged row.
    method!(read_row: fn(at: Index<Row, u16>) -> Option<Row>);
    /// Volts per second the ramp may rise.
    fn set_slew(slew: Fixed<Per<Volt, Second>, One, u32>);
}
