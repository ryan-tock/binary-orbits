//! BFGS refinement of an already-good orbit fit. Used to close the ~0.3%
//! loss gap between Rust's DE (which stops at its convergence criterion)
//! and scipy's DE (which runs L-BFGS-B as a polish step).
//!
//! The parameter vector has 7 entries but only 6 are free — `sm` is
//! recomputed from the weighted least-squares ratio inside calc_loss on
//! every call, so we treat it as dependent and optimize over the other
//! six only. Bounds are enforced by clipping during line search.
//!
//! Gradients are computed by central differences. At 6 free dimensions
//! that's 12 calc_loss evaluations per gradient, ~25µs each on the
//! vectorized path — cheap enough that reverse-mode AD isn't worth
//! pulling in.

use crate::{calc_loss, Params, Point};

pub const NUM_FREE: usize = 6;

#[inline]
fn expand(free: &[f64; NUM_FREE]) -> [f64; 7] {
    [0.0, free[0], free[1], free[2], free[3], free[4], free[5]]
}

#[inline]
fn compress(x: &[f64; 7]) -> [f64; NUM_FREE] {
    [x[1], x[2], x[3], x[4], x[5], x[6]]
}

#[inline]
fn free_bounds(bounds: &[(f64, f64); 7]) -> [(f64, f64); NUM_FREE] {
    [bounds[1], bounds[2], bounds[3], bounds[4], bounds[5], bounds[6]]
}

fn numerical_gradient(
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

#[derive(Clone, Copy, Debug)]
pub struct RefineConfig {
    pub max_iter: usize,
    pub grad_tol: f64,
    pub rel_tol: f64,
}

impl Default for RefineConfig {
    fn default() -> Self {
        Self {
            max_iter: 200,
            grad_tol: 1e-10,
            rel_tol: 1e-12,
        }
    }
}

/// BFGS with backtracking Armijo line search, starting from `x_init`.
///
/// The Hessian inverse approximation `hinv` is kept as a flat [NUM_FREE² ; 36]
/// row-major matrix — negligible overhead at this size and friendlier to the
/// compiler than a boxed matrix type.
pub fn bfgs_refine(
    points: &[Point],
    x_init: &[f64; 7],
    bounds: &[(f64, f64); 7],
    cfg: &RefineConfig,
    scratch: &mut Vec<[f64; 2]>,
) -> ([f64; 7], f64) {
    let fbounds = free_bounds(bounds);
    let mut free = compress(x_init);

    let (mut loss, _) = calc_loss(points, &Params::from_slice(&expand(&free)), scratch);
    let mut g = numerical_gradient(points, &free, &fbounds, scratch);
    let mut hinv = identity();

    for _ in 0..cfg.max_iter {
        // Search direction d = -Hinv · g
        let mut dir = [0.0; NUM_FREE];
        for i in 0..NUM_FREE {
            for j in 0..NUM_FREE {
                dir[i] -= hinv[i * NUM_FREE + j] * g[j];
            }
        }

        // If BFGS gave us a non-descent direction, reset Hinv and fall back to
        // steepest descent for this step.
        let mut g_dot_d: f64 = g.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        if g_dot_d >= 0.0 {
            hinv = identity();
            for i in 0..NUM_FREE {
                dir[i] = -g[i];
            }
            g_dot_d = g.iter().zip(dir.iter()).map(|(a, b)| a * b).sum();
        }

        // Armijo backtracking line search.
        let mut step = 1.0;
        let mut new_free = free;
        let mut new_loss = loss;
        let mut accepted = false;
        for _ in 0..30 {
            for i in 0..NUM_FREE {
                new_free[i] = (free[i] + step * dir[i]).clamp(fbounds[i].0, fbounds[i].1);
            }
            let (tl, _) = calc_loss(points, &Params::from_slice(&expand(&new_free)), scratch);
            if tl <= loss + 1e-4 * step * g_dot_d {
                new_loss = tl;
                accepted = true;
                break;
            }
            step *= 0.5;
        }
        if !accepted {
            break;
        }

        let new_g = numerical_gradient(points, &new_free, &fbounds, scratch);

        // BFGS update of H⁻¹.
        let mut s = [0.0; NUM_FREE];
        let mut y = [0.0; NUM_FREE];
        for i in 0..NUM_FREE {
            s[i] = new_free[i] - free[i];
            y[i] = new_g[i] - g[i];
        }
        let sy: f64 = s.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
        if sy > 1e-10 {
            let rho = 1.0 / sy;
            let mut hy = [0.0; NUM_FREE];
            for i in 0..NUM_FREE {
                for j in 0..NUM_FREE {
                    hy[i] += hinv[i * NUM_FREE + j] * y[j];
                }
            }
            let yhy: f64 = y.iter().zip(hy.iter()).map(|(a, b)| a * b).sum();
            for i in 0..NUM_FREE {
                for j in 0..NUM_FREE {
                    hinv[i * NUM_FREE + j] += (1.0 + rho * yhy) * rho * s[i] * s[j]
                        - rho * (hy[i] * s[j] + s[i] * hy[j]);
                }
            }
        }

        let rel_change = (loss - new_loss).abs() / loss.abs().max(1e-300);
        free = new_free;
        g = new_g;
        loss = new_loss;

        let gnorm: f64 = g.iter().map(|x| x * x).sum::<f64>().sqrt();
        if gnorm < cfg.grad_tol || rel_change < cfg.rel_tol {
            break;
        }
    }

    // Final pass so `sm` (index 0) reflects the LS solution at the converged
    // free parameters.
    let mut x_final = expand(&free);
    let (final_loss, sm) = calc_loss(points, &Params::from_slice(&x_final), scratch);
    x_final[0] = sm;
    (x_final, final_loss)
}

fn identity() -> [f64; NUM_FREE * NUM_FREE] {
    let mut m = [0.0; NUM_FREE * NUM_FREE];
    for i in 0..NUM_FREE {
        m[i * NUM_FREE + i] = 1.0;
    }
    m
}
