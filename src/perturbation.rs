use crate::bla;
use crate::render::{BASE_VIEW_WIDTH, ESCAPE_RADIUS_SQR, Escape};
use crate::{Coordinate, Viewport};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

/// Guard bits beyond the zoom depth, covering image widths up to 2^32 plus rounding slack.
const GUARD_BITS: u32 = 96;

/// The orbit of the view center, iterated in binary fixed point and stored as `f64`, which
/// is enough because every point on it stays within the escape radius.
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

/// A bound on the distance of any pixel from the center: half the view's diagonal, from its
/// half-width and half-height.
fn max_offset(view: &Viewport, aspect: f64) -> f64 {
    BASE_VIEW_WIDTH / view.zoom * (1.0 + aspect) / 2.0
}

pub(crate) fn precision_bits(zoom: f64) -> u32 {
    zoom.log2().ceil().max(0.0) as u32 + GUARD_BITS
}

impl ReferenceOrbit {
    /// The orbit of the view center, with iteration skipping valid for images up to `aspect`
    /// times as tall as they are wide.
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

    /// Whether this orbit can serve `view`: same center and iteration limit, enough
    /// precision for its zoom, and iteration skipping valid over its whole extent.
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

    /// Iterates the pixel at offset `(dcr, dci)` from the center using perturbation theory:
    /// only the difference δ from the reference orbit Z is tracked in `f64`, via
    /// δ' = 2Zδ + δ² + δc. When |Z + δ| < |δ|, or the reference runs out, the pixel is rebased
    /// onto the start of the orbit (δ := Z + δ), which avoids perturbation glitches with a
    /// single reference.
    pub(crate) fn escape<const TRACK_DERIVATIVE: bool>(
        &self,
        dcr: f64,
        dci: f64,
        max_iterations: usize,
    ) -> Option<Escape> {
        self.escape_with::<TRACK_DERIVATIVE, true>(dcr, dci, max_iterations)
    }

    fn escape_with<const TRACK_DERIVATIVE: bool, const SKIP: bool>(
        &self,
        dcr: f64,
        dci: f64,
        max_iterations: usize,
    ) -> Option<Escape> {
        let orbit = &self.points;
        let last = orbit.len() - 1;
        let (mut dr, mut di) = (0.0f64, 0.0f64);
        let (mut der_r, mut der_i) = (1.0f64, 0.0f64);
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
                m += length;
                n += length;
                continue;
            }

            let (ref_r, ref_i) = orbit[m];
            let zr = ref_r + dr;
            let zi = ref_i + di;
            let norm_sqr = zr * zr + zi * zi;
            if norm_sqr > ESCAPE_RADIUS_SQR {
                return Some(Escape {
                    iterations: n,
                    norm_sqr,
                    z: (zr, zi),
                    derivative: rescale(der_r, der_i),
                });
            }
            if TRACK_DERIVATIVE {
                let (d2r, d2i) = (der_r * 2.0, der_i * 2.0);
                (der_r, der_i) = (d2r * zr - d2i * zi + 1.0, d2r * zi + d2i * zr);
            }

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
        None
    }
}

/// Scales by a power of two so the largest component is near 1. Only the direction of the
/// derivative is used, and this keeps its squared norm finite at extreme zooms.
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

    /// Escape iteration of `center + (dx, dy)` iterated entirely in binary fixed point.
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

    /// Checks every pixel of a `size` x `size` grid against exact iteration: plain perturbation
    /// must match exactly, and iteration skipping may change at most `max_skip_mismatches`
    /// pixels. Also checks the grid is not trivially uniform.
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
                let plain = orbit
                    .escape_with::<true, false>(dx, dy, max_iterations)
                    .map(|e| e.iterations);
                let skipped = orbit
                    .escape_with::<true, true>(dx, dy, max_iterations)
                    .map(|e| e.iterations);
                assert_eq!(plain, exact, "pixel ({x}, {y})");
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
        // Just outside the cusp of the main cardioid, where the reference escapes before the
        // pixels around it, forcing them to rebase
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
