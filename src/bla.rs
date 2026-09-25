//! Bivariate linear approximation: while |δ| is below a block's radius, δ² is negligible and
//! the block's iterations collapse into δ → Aδ + B·δc.

const EPSILON: f64 = 1.0 / (1u64 << 24) as f64;

type Complex = (f64, f64);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Step {
    pub(crate) a: Complex,
    pub(crate) b: Complex,
    pub(crate) radius_sqr: f64,
}

#[derive(Debug)]
pub(crate) struct Table {
    // levels[k][j] advances 2^k iterations from reference index 1 + j * 2^k
    levels: Vec<Vec<Step>>,
}

fn mul(x: Complex, y: Complex) -> Complex {
    (x.0 * y.0 - x.1 * y.1, x.0 * y.1 + x.1 * y.0)
}

fn abs(x: Complex) -> f64 {
    x.0.hypot(x.1)
}

impl Table {
    pub(crate) fn new(orbit: &[Complex], max_offset: f64) -> Self {
        // Z_0 = 0 cannot be skipped over, and the last point may already have escaped
        let single: Vec<Step> = orbit[1..orbit.len().saturating_sub(1).max(1)]
            .iter()
            .map(|&z| Step {
                a: (2.0 * z.0, 2.0 * z.1),
                b: (1.0, 0.0),
                radius_sqr: (EPSILON * abs(z)).powi(2),
            })
            .collect();

        let mut levels = vec![single];
        while levels.last().is_some_and(|level| level.len() >= 2) {
            let (pairs, _) = levels.last().unwrap().as_chunks::<2>();
            let merged = pairs.iter().map(|[x, y]| merge(x, y, max_offset)).collect();
            levels.push(merged);
        }
        Self { levels }
    }

    #[cfg(feature = "gpu")]
    pub(crate) fn levels(&self) -> &[Vec<Step>] {
        &self.levels
    }

    pub(crate) fn lookup(&self, m: usize, delta_sqr: f64, budget: usize) -> Option<(&Step, usize)> {
        let index = m.checked_sub(1)?;
        let aligned = (index.trailing_zeros() as usize).min(self.levels.len() - 1);
        let mut found = None;
        // A merged block is never valid further out than its first half
        for level in 0..=aligned {
            let length = 1 << level;
            match self.levels[level].get(index >> level) {
                Some(step) if length <= budget && delta_sqr < step.radius_sqr => {
                    found = Some((step, length))
                }
                _ => break,
            }
        }
        found
    }
}

fn merge(x: &Step, y: &Step, max_offset: f64) -> Step {
    let a = mul(y.a, x.a);
    let b = mul(y.a, x.b);
    let b = (b.0 + y.b.0, b.1 + y.b.1);
    let reach = (y.radius_sqr.sqrt() - abs(x.b) * max_offset) / abs(x.a);
    let radius = if reach.is_finite() {
        reach.max(0.0)
    } else {
        0.0
    };
    Step {
        a,
        b,
        radius_sqr: x.radius_sqr.min(radius * radius),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_steps_compose_the_linear_maps() {
        let orbit = [
            (0.0, 0.0),
            (0.25, 0.5),
            (-0.5, 0.125),
            (0.75, -0.25),
            (1.0, 1.0),
        ];
        let table = Table::new(&orbit, 0.0);
        let (x, y) = (table.levels[0][0], table.levels[0][1]);
        let merged = table.levels[1][0];
        let (delta, dc) = ((1e-9, -2e-9), (3e-10, 1e-10));
        let step = |s: &Step, d: Complex| {
            let ad = mul(s.a, d);
            let bc = mul(s.b, dc);
            (ad.0 + bc.0, ad.1 + bc.1)
        };
        let expected = step(&y, step(&x, delta));
        let actual = step(&merged, delta);
        assert!((expected.0 - actual.0).abs() < 1e-24 && (expected.1 - actual.1).abs() < 1e-24);
    }

    #[test]
    fn lookup_respects_alignment_radius_and_budget() {
        let orbit: Vec<Complex> = (0..20).map(|i| (0.1 * i as f64, 0.2)).collect();
        let table = Table::new(&orbit, 0.0);
        assert!(table.lookup(0, 0.0, 100).is_none());
        let (_, length) = table.lookup(1, 0.0, 100).unwrap();
        assert_eq!(length, 16);
        assert_eq!(table.lookup(1, 0.0, 5).unwrap().1, 4);
        assert_eq!(table.lookup(3, 0.0, 100).unwrap().1, 2);
        assert!(table.lookup(1, 1.0, 100).is_none());
    }

    #[test]
    fn unreachable_blocks_have_zero_radius() {
        let orbit = [(0.0, 0.0), (1.0, 0.0), (1.0, 0.0), (1.0, 0.0)];
        let table = Table::new(&orbit, 1.0);
        assert_eq!(table.levels[1][0].radius_sqr, 0.0);
    }
}
