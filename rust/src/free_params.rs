//! Helpers for the "free parameter" parameterization used by refine + sgd.
//!
//! The full 7-vector [sm, e, i, node, periapsis, m0, p] contains one
//! dependent slot (sm is recomputed from the LS ratio inside calc_loss
//! on every call), so the optimizers work over the remaining 6
//! dimensions. These helpers expand/compress between the two views and
//! provide a shared central-difference gradient. Keeping them here
//! avoids duplicating the same code in both refine.rs and sgd.rs.

use crate::{calc_loss, Params, Point};

pub const NUM_FREE: usize = 6;

/// Free → full: put sm = 0 in slot 0; calc_loss will overwrite it.
#[inline]
pub fn expand(free: &[f64; NUM_FREE]) -> [f64; 7] {
    [0.0, free[0], free[1], free[2], free[3], free[4], free[5]]
}

/// Full → free: drop slot 0.
#[inline]
pub fn compress(x: &[f64; 7]) -> [f64; NUM_FREE] {
    [x[1], x[2], x[3], x[4], x[5], x[6]]
}

/// Pull the 6 free-parameter bounds out of a full 7-vector bounds array.
#[inline]
pub fn free_bounds(bounds: &[(f64, f64); 7]) -> [(f64, f64); NUM_FREE] {
    [bounds[1], bounds[2], bounds[3], bounds[4], bounds[5], bounds[6]]
}

/// Clip each coordinate into its bounds in place.
#[inline]
pub fn clamp(free: &mut [f64; NUM_FREE], bounds: &[(f64, f64); NUM_FREE]) {
    for i in 0..NUM_FREE {
        free[i] = free[i].clamp(bounds[i].0, bounds[i].1);
    }
}

/// Central-difference gradient. Skips dimensions where the ± step would
/// fall fully outside bounds (degenerate `eff_h = 0`).
pub fn gradient(
    points: &[Point],
    free: &[f64; NUM_FREE],
    bounds: &[(f64, f64); NUM_FREE],
    scratch: &mut Vec<[f64; 2]>,
) -> [f64; NUM_FREE] {
    let mut g = [0.0; NUM_FREE];
    for i in 0..NUM_FREE {
        let h = (1e-6_f64 * free[i].abs()).max(1e-8);
        let mut fp = *free;
        let mut fm = *free;
        fp[i] = (free[i] + h).min(bounds[i].1);
        fm[i] = (free[i] - h).max(bounds[i].0);
        let eff_h = fp[i] - fm[i];
        if eff_h <= 0.0 {
            continue;
        }
        let (lp, _) = calc_loss(points, &Params::from_slice(&expand(&fp)), scratch);
        let (lm, _) = calc_loss(points, &Params::from_slice(&expand(&fm)), scratch);
        g[i] = (lp - lm) / eff_h;
    }
    g
}
