//! Differential evolution optimizer. A lean best/1/bin implementation
//! matching the knobs scipy's `differential_evolution` uses by default
//! (popsize 15×dim, dithered mutation, crossover 0.7, best1bin, stddev/mean
//! convergence with tol=0.01, init=latinhypercube). We don't try to match
//! scipy's exact RNG behavior bit-for-bit — just the convergence behavior.

use fastrand::Rng;

/// Configuration. Defaults match scipy's `differential_evolution` defaults.
#[derive(Clone, Copy, Debug)]
pub struct DeConfig {
    pub popsize_mult: usize,
    pub max_iter: usize,
    pub f_range: (f64, f64),
    pub cr: f64,
    pub tol: f64,
    pub seed: u64,
}

impl Default for DeConfig {
    fn default() -> Self {
        Self {
            popsize_mult: 15,
            max_iter: 1000,
            f_range: (0.5, 1.0),
            cr: 0.7,
            tol: 0.01,
            seed: 0xDEAD_BEEF_BADDF00D,
        }
    }
}

/// Draw a Latin-hypercube initial population: for each dimension, split
/// [lo, hi] into `popsize` equal bins, then permute which bin each
/// member lands in. Lifted from the DE literature — matches scipy's
/// `init='latinhypercube'` default, which tends to be a bit more stable
/// than uniform sampling.
fn latin_hypercube(rng: &mut Rng, bounds: &[(f64, f64)], popsize: usize) -> Vec<Vec<f64>> {
    let dim = bounds.len();
    let mut pop = vec![vec![0.0; dim]; popsize];
    for d in 0..dim {
        let (lo, hi) = bounds[d];
        let span = hi - lo;
        // For each member, sample uniformly from its assigned bin.
        let mut bins: Vec<usize> = (0..popsize).collect();
        rng.shuffle(&mut bins);
        for (i, &b) in bins.iter().enumerate() {
            let t = (b as f64 + rng.f64()) / popsize as f64;
            pop[i][d] = lo + t * span;
        }
    }
    pop
}

/// Minimize `loss` over `bounds` using best1bin DE.
///
/// Each trial gets exactly one `loss` call. Returns (best parameters,
/// best energy, actual iterations run).
pub fn differential_evolution<F: FnMut(&[f64]) -> f64>(
    bounds: &[(f64, f64)],
    mut loss: F,
    cfg: &DeConfig,
) -> (Vec<f64>, f64, usize) {
    let dim = bounds.len();
    let popsize = dim * cfg.popsize_mult;
    let mut rng = Rng::with_seed(cfg.seed);

    let mut population = latin_hypercube(&mut rng, bounds, popsize);
    let mut energies: Vec<f64> = population.iter().map(|x| loss(x)).collect();

    let mut best_idx = 0usize;
    for i in 1..popsize {
        if energies[i] < energies[best_idx] {
            best_idx = i;
        }
    }

    let mut iters = 0;
    for _gen in 0..cfg.max_iter {
        iters += 1;
        // Dither the mutation factor once per generation, like scipy does.
        let f = cfg.f_range.0 + rng.f64() * (cfg.f_range.1 - cfg.f_range.0);

        for i in 0..popsize {
            // Pick two distinct indices b, c, both != i and != best_idx.
            // (best1bin uses best_idx as the base; we still need b != c.)
            let (b, c) = loop {
                let b = rng.usize(..popsize);
                let c = rng.usize(..popsize);
                if b != i && c != i && b != c && b != best_idx && c != best_idx {
                    break (b, c);
                }
            };

            let force = rng.usize(..dim);
            let mut trial = population[i].clone();
            for d in 0..dim {
                if d == force || rng.f64() < cfg.cr {
                    let mutant = population[best_idx][d] + f * (population[b][d] - population[c][d]);
                    trial[d] = mutant.clamp(bounds[d].0, bounds[d].1);
                }
            }

            let trial_energy = loss(&trial);
            if trial_energy < energies[i] {
                population[i] = trial;
                energies[i] = trial_energy;
                if trial_energy < energies[best_idx] {
                    best_idx = i;
                }
            }
        }

        // Convergence: stddev / |mean| of energies < tol. Matches scipy.
        let mean: f64 = energies.iter().sum::<f64>() / popsize as f64;
        let var: f64 = energies.iter().map(|&e| (e - mean).powi(2)).sum::<f64>() / popsize as f64;
        let stddev = var.sqrt();
        if mean.abs() > 0.0 && stddev / mean.abs() < cfg.tol {
            break;
        }
    }

    let best = population[best_idx].clone();
    (best, energies[best_idx], iters)
}
