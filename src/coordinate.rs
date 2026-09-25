use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{Signed, Zero};
use std::fmt;
use std::str::FromStr;

/// Longest decimal expansion accepted, which bounds the work any single coordinate can cause.
pub const MAX_DIGITS: u32 = 2000;

/// An exact decimal number `digits / 10^scale`, used for view centers deeper than `f64` can
/// address. Always kept normalized (no trailing fractional zeros), so equality is structural.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Coordinate {
    digits: BigInt,
    scale: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCoordinateError(&'static str);

impl fmt::Display for ParseCoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for ParseCoordinateError {}

impl Coordinate {
    fn new(digits: BigInt, scale: u32) -> Self {
        let mut coordinate = Self { digits, scale };
        let ten = BigInt::from(10);
        while coordinate.scale > 0 && coordinate.digits.is_multiple_of(&ten) {
            coordinate.digits /= &ten;
            coordinate.scale -= 1;
        }
        if coordinate.digits.is_zero() {
            coordinate.scale = 0;
        }
        coordinate
    }

    /// The shortest decimal that converts back to exactly `value`.
    pub fn from_f64(value: f64) -> Self {
        assert!(value.is_finite(), "coordinates must be finite");
        format!("{value}")
            .parse()
            .expect("f64 formats as a decimal")
    }

    /// The nearest `f64`.
    pub fn to_f64(&self) -> f64 {
        self.to_string()
            .parse()
            .expect("coordinates format as decimals")
    }

    /// Decimal places after the point.
    pub fn scale(&self) -> u32 {
        self.scale
    }

    /// `self + delta`, rounded to `scale` decimal places.
    pub fn offset(&self, delta: f64, scale: u32) -> Self {
        assert!(delta.is_finite(), "offsets must be finite");
        let scale = scale.min(MAX_DIGITS);
        let base = if scale >= self.scale {
            &self.digits * pow10(scale - self.scale)
        } else {
            div_round(&self.digits, &pow10(self.scale - scale))
        };
        Self::new(base + scaled_f64(delta, scale), scale)
    }

    /// `self - other` as the nearest `f64`.
    pub fn difference(&self, other: &Self) -> f64 {
        let scale = self.scale.max(other.scale);
        let diff =
            &self.digits * pow10(scale - self.scale) - &other.digits * pow10(scale - other.scale);
        Self::new(diff, scale).to_f64()
    }

    /// The value in binary fixed point with `frac_bits` fractional bits, rounded to nearest.
    pub(crate) fn to_fixed(&self, frac_bits: u32) -> BigInt {
        div_round(&(&self.digits << frac_bits as usize), &pow10(self.scale))
    }
}

fn pow10(exponent: u32) -> BigInt {
    num_traits::pow(BigInt::from(10), exponent as usize)
}

fn div_round(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let (quotient, remainder) = numerator.div_mod_floor(denominator);
    if (remainder << 1usize) >= *denominator {
        quotient + 1
    } else {
        quotient
    }
}

/// `round(value * 10^scale)`, computed exactly from the binary representation of `value`.
fn scaled_f64(value: f64, scale: u32) -> BigInt {
    if value == 0.0 {
        return BigInt::zero();
    }
    let bits = value.abs().to_bits();
    let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, exponent) = if raw_exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), raw_exponent - 1075)
    };
    let scaled = BigInt::from(mantissa) * pow10(scale);
    let magnitude = if exponent >= 0 {
        scaled << exponent as usize
    } else {
        div_round(&scaled, &(BigInt::from(1) << (-exponent) as usize))
    };
    if value < 0.0 { -magnitude } else { magnitude }
}

impl FromStr for Coordinate {
    type Err = ParseCoordinateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (negative, unsigned) = match s.as_bytes().first() {
            Some(b'-') => (true, &s[1..]),
            Some(b'+') => (false, &s[1..]),
            _ => (false, s),
        };
        let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
            Some((mantissa, exponent)) => (
                mantissa,
                exponent
                    .parse::<i64>()
                    .map_err(|_| ParseCoordinateError("invalid exponent"))?,
            ),
            None => (unsigned, 0),
        };
        let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let all_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        if integer.is_empty() && fraction.is_empty()
            || !all_digits(integer)
            || !all_digits(fraction)
        {
            return Err(ParseCoordinateError("expected a decimal number"));
        }

        let scale = fraction.len() as i64 - exponent;
        let length = (integer.len() + fraction.len()) as i64;
        if scale.abs() > MAX_DIGITS as i64 || length > MAX_DIGITS as i64 {
            return Err(ParseCoordinateError("too many digits"));
        }
        let magnitude: BigInt = format!("0{integer}{fraction}")
            .parse()
            .expect("validated digits");
        let (magnitude, scale) = if scale < 0 {
            (magnitude * pow10(scale.unsigned_abs() as u32), 0)
        } else {
            (magnitude, scale as u32)
        };
        Ok(Self::new(
            if negative { -magnitude } else { magnitude },
            scale,
        ))
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let digits = self.digits.abs().to_string();
        let sign = if self.digits.sign() == Sign::Minus {
            "-"
        } else {
            ""
        };
        let scale = self.scale as usize;
        if scale == 0 {
            return write!(f, "{sign}{digits}");
        }
        let padded = format!("{digits:0>width$}", width = scale + 1);
        let (integer, fraction) = padded.split_at(padded.len() - scale);
        write!(f, "{sign}{integer}.{fraction}")
    }
}

impl From<f64> for Coordinate {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(s: &str) -> Coordinate {
        s.parse().unwrap()
    }

    #[test]
    fn parses_and_formats_decimals() {
        for (input, expected) in [
            ("0", "0"),
            ("-0.0", "0"),
            ("1.2500", "1.25"),
            ("-1.249559196", "-1.249559196"),
            ("+.5", "0.5"),
            ("7.", "7"),
            ("-4.621603e-1", "-0.4621603"),
            ("1.5E3", "1500"),
            ("12e-5", "0.00012"),
        ] {
            assert_eq!(c(input).to_string(), expected, "{input}");
        }
    }

    #[test]
    fn rejects_invalid_input() {
        for input in [
            "", "-", ".", "1.2.3", "abc", "1e", "1e5x", "--1", "1,5", "1e999999",
        ] {
            assert!(input.parse::<Coordinate>().is_err(), "{input}");
        }
    }

    #[test]
    fn f64_round_trips_exactly() {
        for value in [
            0.0,
            -0.75,
            0.030466443,
            -1.2494989,
            1e-300,
            -2.5e-12,
            123456.789,
            f64::MIN_POSITIVE,
            5e-324,
        ] {
            assert_eq!(Coordinate::from_f64(value).to_f64(), value);
        }
    }

    #[test]
    fn to_f64_is_correctly_rounded() {
        let long = "-1.24955919600000000000000000000000000000017";
        assert_eq!(c(long).to_f64(), long.parse::<f64>().unwrap());
    }

    #[test]
    fn offset_is_exact_at_requested_scale() {
        let base = c("-1.249559196000000000000000000001");
        assert_eq!(
            base.offset(0.5, 30).to_string(),
            "-0.749559196000000000000000000001"
        );
        assert_eq!(
            c("1").offset(2f64.powi(-40), 12).to_string(),
            "1.000000000001"
        );
        assert_eq!(
            c("0").offset(-1e-30, 35).to_string(),
            "-0.000000000000000000000000000001"
        );
        assert_eq!(
            c("0.1").difference(&c("0.1000000000000000000000000000000000001")),
            -1e-37
        );
    }

    #[test]
    fn converts_to_binary_fixed_point() {
        assert_eq!(c("0.75").to_fixed(4), BigInt::from(12));
        assert_eq!(c("-0.75").to_fixed(4), BigInt::from(-12));
        assert_eq!(c("0.1").to_fixed(10), BigInt::from(102));
    }
}
