use crate::bla;
use crate::render::{BASE_VIEW_WIDTH, ESCAPE_RADIUS_SQR, Escape, Outcome};
use crate::{Coordinate, Viewport};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

const GUARD_BITS: u32 = 96;
const INTERIOR_CONTRACTION: f64 = 1e-6;
const CONTRACTION_CAP: f64 = 1e100;

#[derive(Debug)]
pub struct ReferenceOrbit {
    center_x: Coordinate,
    center_y: Coordinate,
    max_iterations: usize,
    frac_bits: u32,
    points: Vec<(f64, f64)>,
    max_offset: f64,
    bla: bla::Table,
}

fn max_offset(view: &Viewport, aspect: f64) -> f64 {
    BASE_VIEW_WIDTH / view.zoom * (1.0 + aspect) / 2.0
}

pub(crate) fn precision_bits(zoom: f64) -> u32 {
    zoom.log2().ceil().max(0.0) as u32 + GUARD_BITS
}

impl ReferenceOrbit {
    pub fn compute(view: &Viewport, max_iterations: usize, aspect: f64) -> Self {
        let frac_bits = precision_bits(view.zoom);
        let cr = view.center_x.to_fixed(frac_bits);
        let ci = view.center_y.to_fixed(frac_bits);
        let shift = frac_bits as usize;

        let mut points = Vec::with_capacity(max_iterations + 1);
        let (mut zr, mut zi) = (BigInt::ZERO, BigInt::ZERO);
        points.push((0.0, 0.0));
        for _ in 0..max_iterations {
            let zr2 = (&zr * &zr) >> shift;
            let zi2 = (&zi * &zi) >> shift;
            let zri = (&zr * &zi) >> (shift - 1);
            zr = zr2 - zi2 + &cr;
            zi = zri + &ci;

            let point = (fixed_to_f64(&zr, frac_bits), fixed_to_f64(&zi, frac_bits));
            points.push(point);
            if point.0 * point.0 + point.1 * point.1 > ESCAPE_RADIUS_SQR {
                break;
            }
        }

        let max_offset = max_offset(view, aspect);
        Self {
            center_x: view.center_x.clone(),
            center_y: view.center_y.clone(),
            max_iterations,
            frac_bits,
            bla: bla::Table::new(&points, max_offset),
            points,
            max_offset,
        }
    }

    pub fn matches(&self, view: &Viewport, max_iterations: usize, aspect: f64) -> bool {
        self.center_x == view.center_x
            && self.center_y == view.center_y
            && self.max_iterations == max_iterations
            && self.frac_bits >= precision_bits(view.zoom)
            && self.max_offset >= max_offset(view, aspect)
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    #[cfg(feature = "gpu")]
    pub(crate) fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    pub(crate) fn escape<const TRACK_DERIVATIVE: bool>(
        &self,
        dcr: f64,
        dci: f64,
        max_iterations: usize,
    ) -> Option<Escape> {
        match self.iterate::<TRACK_DERIVATIVE, true, false>(dcr, dci, max_iterations) {
            Outcome::Escaped(escape) => Some(escape),
            _ => None,
        }
    }

    #[cfg(all(test, feature = "gpu"))]
    pub(crate) fn escape_unskipped(
        &self,
        dcr: f64,
        dci: f64,
        max_iterations: usize,
    ) -> Option<usize> {
        match self.iterate::<false, false, false>(dcr, dci, max_iterations) {
            Outcome::Escaped(escape) => Some(escape.iterations),
            _ => None,
        }
    }

    pub(crate) fn classify(&self, dcr: f64, dci: f64, max_iterations: usize) -> Outcome {
        self.iterate::<false, true, true>(dcr, dci, max_iterations)
    }

    fn iterate<const TRACK_DERIVATIVE: bool, const SKIP: bool, const DETECT_INTERIOR: bool>(
        &self,
        dcr: f64,
        dci: f64,
        max_iterations: usize,
    ) -> Outcome {
        let orbit = &self.points;
        let last = orbit.len() - 1;
        let (mut dr, mut di) = (0.0f64, 0.0f64);
        let (mut der_r, mut der_i) = (1.0f64, 0.0f64);
        let mut contraction = Contraction::new();
        let mut m = 0;
        let mut n = 0;

        while n < max_iterations {
            let skip = SKIP
                .then(|| self.bla.lookup(m, dr * dr + di * di, max_iterations - n))
                .flatten();
            if let Some((step, length)) = skip {
                (dr, di) = (
                    step.a.0 * dr - step.a.1 * di + step.b.0 * dcr - step.b.1 * dci,
                    step.a.0 * di + step.a.1 * dr + step.b.0 * dci + step.b.1 * dcr,
                );
                if TRACK_DERIVATIVE {
                    (der_r, der_i) = (
                        step.a.0 * der_r - step.a.1 * der_i + step.b.0,
                        step.a.0 * der_i + step.a.1 * der_r + step.b.1,
                    );
                }
                if DETECT_INTERIOR && contraction.multiply(step.a) {
                    return Outcome::Interior;
                }
                m += length;
                n += length;
                continue;
            }

            let (ref_r, ref_i) = orbit[m];
            let zr = ref_r + dr;
            let zi = ref_i + di;
            let norm_sqr = zr * zr + zi * zi;
            if norm_sqr > ESCAPE_RADIUS_SQR {
                return Outcome::Escaped(Escape {
                    iterations: n,
                    norm_sqr,
                    z: (zr, zi),
                    derivative: rescale(der_r, der_i),
                });
            }
            if DETECT_INTERIOR && n > 0 && contraction.multiply((2.0 * zr, 2.0 * zi)) {
                return Outcome::Interior;
            }
            if TRACK_DERIVATIVE {
                let (d2r, d2i) = (der_r * 2.0, der_i * 2.0);
                (der_r, der_i) = (d2r * zr - d2i * zi + 1.0, d2r * zi + d2i * zr);
            }

            // Rebasing onto the start of the orbit avoids glitches with a single reference
            if m == last || norm_sqr < dr * dr + di * di {
                (dr, di) = (zr, zi);
                m = 0;
            }
            let (ref_r, ref_i) = orbit[m];
            (dr, di) = (
                2.0 * (ref_r * dr - ref_i * di) + (dr * dr - di * di) + dcr,
                2.0 * (ref_r * di + ref_i * dr) + 2.0 * dr * di + dci,
            );
            m += 1;
            n += 1;
        }
        Outcome::Undecided
    }
}

/// The product of 2z along an orbit, from z_1: it shrinks towards zero only when the orbit is
/// drawn into an attracting cycle, i.e. the point is inside the set.
pub(crate) struct Contraction(f64, f64);

impl Contraction {
    pub(crate) fn new() -> Self {
        Self(1.0, 0.0)
    }

    /// Multiplies in `factor`, returning whether the orbit is now known to be interior.
    pub(crate) fn multiply(&mut self, factor: (f64, f64)) -> bool {
        let (r, i) = (
            self.0 * factor.0 - self.1 * factor.1,
            self.0 * factor.1 + self.1 * factor.0,
        );
        let norm_sqr = r * r + i * i;
        let scale = if norm_sqr > CONTRACTION_CAP * CONTRACTION_CAP {
            CONTRACTION_CAP / norm_sqr.sqrt()
        } else {
            1.0
        };
        (self.0, self.1) = (r * scale, i * scale);
        norm_sqr < INTERIOR_CONTRACTION * INTERIOR_CONTRACTION
    }
}

/// Only the derivative's direction is used; scaling keeps its squared norm finite.
fn rescale(x: f64, y: f64) -> (f64, f64) {
    let largest = x.abs().max(y.abs());
    if largest == 0.0 || !largest.is_finite() {
        return (x, y);
    }
    let scale = 2f64.powi(-(largest.log2().floor() as i32));
    (x * scale, y * scale)
}

fn fixed_to_f64(value: &BigInt, frac_bits: u32) -> f64 {
    let shift = value.bits().saturating_sub(64);
    let top = (value >> shift as usize)
        .to_f64()
        .expect("64-bit values fit in f64");
    let exponent = shift as i32 - frac_bits as i32;
    top * 2f64.powi(exponent / 2) * 2f64.powi(exponent - exponent / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iterations(outcome: Outcome) -> Option<usize> {
        match outcome {
            Outcome::Escaped(escape) => Some(escape.iterations),
            _ => None,
        }
    }

    fn exact_escape(view: &Viewport, dx: f64, dy: f64, max_iterations: usize) -> Option<usize> {
        let frac_bits = precision_bits(view.zoom) + 64;
        let scale = (view.zoom.log10() as u32) + 40;
        let cr = view.center_x.offset(dx, scale).to_fixed(frac_bits);
        let ci = view.center_y.offset(dy, scale).to_fixed(frac_bits);
        let shift = frac_bits as usize;
        let (mut zr, mut zi) = (BigInt::ZERO, BigInt::ZERO);
        for n in 0..max_iterations {
            let (r, i) = (fixed_to_f64(&zr, frac_bits), fixed_to_f64(&zi, frac_bits));
            if r * r + i * i > ESCAPE_RADIUS_SQR {
                return Some(n);
            }
            let zri = (&zr * &zi) >> (shift - 1);
            zr = ((&zr * &zr) >> shift) - ((&zi * &zi) >> shift) + &cr;
            zi = zri + &ci;
        }
        None
    }

    fn assert_matches_exact(
        view: &Viewport,
        size: usize,
        max_iterations: usize,
        max_skip_mismatches: usize,
    ) {
        let orbit = ReferenceOrbit::compute(view, max_iterations, 1.0);
        let pixel = 3.0 / view.zoom / size as f64;
        let mut distinct = std::collections::HashSet::new();
        let mut skip_mismatches = 0;
        for y in 0..size {
            for x in 0..size {
                let dx = (x as f64 - size as f64 / 2.0) * pixel;
                let dy = (y as f64 - size as f64 / 2.0) * pixel;
                let exact = exact_escape(view, dx, dy, max_iterations);
                let plain = iterations(orbit.iterate::<true, false, false>(dx, dy, max_iterations));
                let skipped =
                    iterations(orbit.iterate::<true, true, false>(dx, dy, max_iterations));
                assert_eq!(plain, exact, "pixel ({x}, {y})");
                if let Outcome::Interior = orbit.classify(dx, dy, max_iterations) {
                    assert_eq!(
                        exact, None,
                        "pixel ({x}, {y}) wrongly classified as interior"
                    );
                }
                skip_mismatches += (skipped != exact) as usize;
                distinct.insert(exact);
            }
        }
        eprintln!(
            "zoom {:e}: {skip_mismatches}/{} skipped mismatches",
            view.zoom,
            size * size
        );
        assert!(
            skip_mismatches <= max_skip_mismatches,
            "{skip_mismatches} pixels changed by skipping"
        );
        assert!(distinct.len() > 1, "grid should not be uniform");
    }

    #[test]
    fn matches_exact_iteration_at_deep_zoom() {
        let view = Viewport {
            center_x: "0".parse().unwrap(),
            center_y: "1".parse().unwrap(),
            zoom: 1e30,
        };
        assert_matches_exact(&view, 24, 3000, 0);
    }

    #[test]
    fn matches_exact_iteration_at_extreme_zoom() {
        let view = Viewport {
            center_x: "0".parse().unwrap(),
            center_y: "1".parse().unwrap(),
            zoom: 1e100,
        };
        assert_matches_exact(&view, 16, 3000, 0);
    }

    #[test]
    fn matches_exact_iteration_when_reference_escapes_early() {
        // Just outside the cardioid cusp, the reference escapes first and pixels must rebase
        let view = Viewport {
            center_x: "0.2501".parse().unwrap(),
            center_y: "0".parse().unwrap(),
            zoom: 1e3,
        };
        assert!(ReferenceOrbit::compute(&view, 2000, 1.0).len() < 2000);
        assert_matches_exact(&view, 24, 2000, 0);
    }

    #[test]
    fn matches_exact_iteration_near_minibrot() {
        let view = Viewport::from_f64(-1.249559196, 0.030466443, 1.73e6);
        assert_matches_exact(&view, 24, 1500, 576 / 100);
    }

    #[test]
    fn fixed_point_converts_to_nearest_f64() {
        assert_eq!(fixed_to_f64(&BigInt::from(3), 1), 1.5);
        assert_eq!(fixed_to_f64(&BigInt::from(-3), 2), -0.75);
        let third = (BigInt::from(1) << 300usize) / 3;
        assert!((fixed_to_f64(&third, 300) - 1.0 / 3.0).abs() < 1e-16);
    }

    #[test]
    fn orbit_of_interior_point_runs_to_the_limit() {
        let orbit = ReferenceOrbit::compute(&Viewport::from_f64(-0.1, 0.1, 1e12), 500, 1.0);
        assert_eq!(orbit.len(), 501);
    }

    #[test]
    fn orbit_of_exterior_point_stops_after_escaping() {
        let orbit = ReferenceOrbit::compute(&Viewport::from_f64(0.5, 0.5, 1e12), 500, 1.0);
        assert!(orbit.len() < 20);
        let (x, y) = orbit.points[orbit.len() - 1];
        assert!(x * x + y * y > ESCAPE_RADIUS_SQR);
    }

    #[test]
    fn matches_requires_same_center_and_coverage() {
        let view = Viewport::from_f64(-0.1, 0.1, 1e12);
        let orbit = ReferenceOrbit::compute(&view, 500, 1.0);
        assert!(orbit.matches(&view, 500, 1.0));
        assert!(orbit.matches(&view, 500, 0.5));
        assert!(!orbit.matches(&view, 500, 2.0));
        let wider = Viewport {
            zoom: 1e11,
            ..view.clone()
        };
        assert!(!orbit.matches(&wider, 500, 1.0));
        let deeper = Viewport {
            zoom: 1e40,
            ..view.clone()
        };
        assert!(!orbit.matches(&deeper, 500, 1.0));
        assert!(!orbit.matches(&view, 501, 1.0));
        assert!(!orbit.matches(&Viewport::from_f64(-0.1, 0.2, 1e12), 500, 1.0));
    }
}
