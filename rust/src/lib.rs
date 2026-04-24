//! Rust port of the binary-orbits inner math.
//!
//! The Python reference (`server.py::calc_positions`, `calc_loss`) is the
//! source of truth for accuracy. Unit tests in `tests/parity.rs` lock us
//! to it within float tolerance; the criterion benches in `benches/` cover
//! speed. Tracing features live behind `trace-iter`, `trace-point`, and
//! `trace-hot` so they cost nothing in release builds.

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyDict, PyList};

#[cfg(any(feature = "trace-iter", feature = "trace-point", feature = "trace-hot"))]
use std::time::Instant;

pub mod de;

pub const NEWTON_ITERATIONS: usize = 6;

// Tracing macros: zero overhead when the relevant feature is off.
macro_rules! trace_iter {
    ($($arg:tt)*) => {
        #[cfg(feature = "trace-iter")]
        eprintln!("[trace-iter] {}", format!($($arg)*));
    };
}
macro_rules! trace_point {
    ($($arg:tt)*) => {
        #[cfg(feature = "trace-point")]
        eprintln!("[trace-point] {}", format!($($arg)*));
    };
}
macro_rules! trace_hot {
    ($($arg:tt)*) => {
        #[cfg(feature = "trace-hot")]
        eprintln!("[trace-hot] {}", format!($($arg)*));
    };
}

/// Single observation: epoch, x, y, weight. We keep the field layout here
/// — outside PyO3 — so the benches/tests can drive the math without Python.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub weight: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub sm: f64,
    pub e: f64,
    pub i: f64,
    pub node: f64,
    pub periapsis: f64,
    pub m0: f64,
    pub p: f64,
}

impl Params {
    pub fn from_slice(s: &[f64]) -> Self {
        Self {
            sm: s[0],
            e: s[1],
            i: s[2],
            node: s[3],
            periapsis: s[4],
            m0: s[5],
            p: s[6],
        }
    }
}

/// Mirror of `calc_positions` in server.py. Fills `out` with (x, y) pairs
/// per point. The semi-major axis is *not* applied here; callers scale by
/// `sm` afterwards.
#[inline]
pub fn calc_positions(points: &[Point], params: &Params, out: &mut Vec<[f64; 2]>) {
    #[cfg(feature = "trace-iter")]
    let _iter_t = Instant::now();

    out.clear();
    out.reserve(points.len());

    let node_shifted = params.node - 3.0 * std::f64::consts::PI / 2.0;
    let node_cos = node_shifted.cos();
    let node_sin = node_shifted.sin();
    let inclined_angle = params.i.cos();
    let beta = params.e / (1.0 + (1.0 - params.e * params.e).sqrt());
    let two_pi_over_p = 2.0 * std::f64::consts::PI / params.p;

    for (_idx, point) in points.iter().enumerate() {
        #[cfg(feature = "trace-point")]
        let _pt_t = Instant::now();

        let mean_anomaly = params.m0 + two_pi_over_p * (point.t - 2000.0);

        #[cfg(feature = "trace-hot")]
        let _newton_t = Instant::now();
        let mut eccentric_anomaly = mean_anomaly;
        for _ in 0..NEWTON_ITERATIONS {
            let (sin_e, cos_e) = eccentric_anomaly.sin_cos();
            eccentric_anomaly += (mean_anomaly - eccentric_anomaly + params.e * sin_e)
                / (1.0 - params.e * cos_e);
        }
        trace_hot!("newton iterations took {:?}", _newton_t.elapsed());

        let (sin_e, cos_e) = eccentric_anomaly.sin_cos();
        let true_anomaly = eccentric_anomaly
            + 2.0 * (beta * sin_e / (1.0 - beta * cos_e)).atan();

        let r = params.sm * (1.0 - params.e * cos_e);
        let (sin_w, cos_w) = (true_anomaly + params.periapsis).sin_cos();

        out.push([
            r * (cos_w * node_cos - inclined_angle * sin_w * node_sin),
            r * (inclined_angle * sin_w * node_cos + cos_w * node_sin),
        ]);

        trace_point!("point idx={} t={} took {:?}", _idx, point.t, _pt_t.elapsed());
    }

    trace_iter!("calc_positions n={} took {:?}", points.len(), _iter_t.elapsed());
}

/// Mirror of `calc_loss`. Returns both the loss and the least-squares
/// optimal semi-major axis so the caller can splice it back into the
/// parameter vector after DE finishes. `params.sm` is ignored on input
/// (matches the Python original, which overwrites it internally).
#[inline]
pub fn calc_loss(points: &[Point], params: &Params, scratch: &mut Vec<[f64; 2]>) -> (f64, f64) {
    #[cfg(feature = "trace-iter")]
    let _iter_t = Instant::now();

    // Compute positions with unit semi-major; the actual SM drops out of
    // the weighted least-squares ratio below.
    let unit_params = Params { sm: 1.0, ..*params };
    calc_positions(points, &unit_params, scratch);

    #[cfg(feature = "trace-hot")]
    let _ls_t = Instant::now();
    let mut parameter_squared = 0.0;
    let mut resultant = 0.0;
    for (point, pred) in points.iter().zip(scratch.iter()) {
        parameter_squared += (pred[0] * pred[0] + pred[1] * pred[1]) * point.weight;
        resultant += (pred[0] * point.x + pred[1] * point.y) * point.weight;
    }

    let sm = if parameter_squared > 0.0 {
        (resultant / parameter_squared).max(0.0)
    } else {
        0.0
    };
    trace_hot!("LS regression took {:?}", _ls_t.elapsed());

    #[cfg(feature = "trace-hot")]
    let _err_t = Instant::now();
    let mut error = 0.0;
    for (point, pred) in points.iter().zip(scratch.iter()) {
        let dx = point.x - sm * pred[0];
        let dy = point.y - sm * pred[1];
        error += (dx * dx + dy * dy) * point.weight;
    }
    trace_hot!("error sum took {:?}", _err_t.elapsed());

    trace_iter!("calc_loss n={} loss={:.6e} sm={:.6e} took {:?}",
        points.len(), error, sm, _iter_t.elapsed());

    (error, sm)
}

// --- PyO3 bindings (gated on the `python` feature) ---

#[cfg(feature = "python")]
fn points_from_pylist(data: &Bound<'_, PyList>) -> PyResult<Vec<Point>> {
    let mut out = Vec::with_capacity(data.len());
    for item in data.iter() {
        let d = item.downcast::<PyDict>()?;
        let t: f64 = d.get_item("t")?.unwrap().extract()?;
        let x: f64 = d.get_item("x")?.unwrap().extract()?;
        let y: f64 = d.get_item("y")?.unwrap().extract()?;
        let weight: f64 = d.get_item("weight")?.unwrap().extract()?;
        out.push(Point { t, x, y, weight });
    }
    Ok(out)
}

/// Precomputed dataset owned on the Rust side. Build one per input
/// dataset, then pass it to `calc_loss` / `optimal_sm` inside the DE loop
/// to skip the per-call dict→Vec conversion (which otherwise dominates
/// the FFI cost for datasets with hundreds of points).
#[cfg(feature = "python")]
#[pyclass]
pub struct Dataset {
    pub(crate) points: Vec<Point>,
    pub(crate) scratch: std::cell::RefCell<Vec<[f64; 2]>>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Dataset {
    #[new]
    fn new(data: Bound<'_, PyList>) -> PyResult<Self> {
        let points = points_from_pylist(&data)?;
        let cap = points.len();
        Ok(Dataset {
            points,
            scratch: std::cell::RefCell::new(Vec::with_capacity(cap)),
        })
    }

    #[getter]
    fn n(&self) -> usize {
        self.points.len()
    }
}

/// Python: `calc_loss(parameters, data) -> float`
///
/// `data` accepts either a `Dataset` (fast path — no per-call conversion)
/// or a plain `list[dict]` (slow path, still supported for ad-hoc use).
/// Returns the loss only — matches what scipy's differential_evolution
/// wants from its objective.
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calc_loss")]
fn calc_loss_py(parameters: Vec<f64>, data: Bound<'_, PyAny>) -> PyResult<f64> {
    let params = Params::from_slice(&parameters);

    if let Ok(ds) = data.extract::<PyRef<'_, Dataset>>() {
        let mut scratch = ds.scratch.borrow_mut();
        let (loss, _) = calc_loss(&ds.points, &params, &mut scratch);
        return Ok(loss);
    }

    let list = data.downcast::<PyList>()?;
    let points = points_from_pylist(list)?;
    let mut scratch = Vec::with_capacity(points.len());
    let (loss, _) = calc_loss(&points, &params, &mut scratch);
    Ok(loss)
}

/// Python: `optimal_sm(parameters, data) -> float` — returns the LS-optimal
/// semi-major axis for a given parameter vector (ignores parameters[0]).
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "optimal_sm")]
fn optimal_sm_py(parameters: Vec<f64>, data: Bound<'_, PyAny>) -> PyResult<f64> {
    let params = Params::from_slice(&parameters);

    if let Ok(ds) = data.extract::<PyRef<'_, Dataset>>() {
        let mut scratch = ds.scratch.borrow_mut();
        let (_, sm) = calc_loss(&ds.points, &params, &mut scratch);
        return Ok(sm);
    }

    let list = data.downcast::<PyList>()?;
    let points = points_from_pylist(list)?;
    let mut scratch = Vec::with_capacity(points.len());
    let (_, sm) = calc_loss(&points, &params, &mut scratch);
    Ok(sm)
}

/// Python: `calc_positions(parameters, data) -> list[list[float]]`
///
/// Returned pairs are raw (sm=parameters[0] is applied), matching the
/// Python function's output shape.
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "calc_positions")]
fn calc_positions_py(
    parameters: Vec<f64>,
    data: Bound<'_, PyAny>,
) -> PyResult<Vec<[f64; 2]>> {
    let params = Params::from_slice(&parameters);
    let mut out = Vec::new();

    if let Ok(ds) = data.extract::<PyRef<'_, Dataset>>() {
        out.reserve(ds.points.len());
        calc_positions(&ds.points, &params, &mut out);
        return Ok(out);
    }

    let list = data.downcast::<PyList>()?;
    let points = points_from_pylist(list)?;
    out.reserve(points.len());
    calc_positions(&points, &params, &mut out);
    Ok(out)
}

/// Python: `fit_orbit(dataset, period_bound, *, seed=None) -> list[float]`
///
/// End-to-end fit entirely in Rust: runs best1bin differential evolution
/// over the seven orbital parameters (sm is pinned to 0 within DE; the LS
/// optimum is spliced in after). Skips scipy entirely, so no Python calls
/// inside the hot loop at all.
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "fit_orbit", signature = (dataset, period_bound, seed=None, max_iter=1000))]
fn fit_orbit_py(
    dataset: PyRef<'_, Dataset>,
    period_bound: (f64, f64),
    seed: Option<u64>,
    max_iter: usize,
) -> PyResult<Vec<f64>> {
    let bounds: [(f64, f64); 7] = [
        (0.0, 0.0),
        (0.0, 0.95),
        (0.0, std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        (0.0, 2.0 * std::f64::consts::PI),
        period_bound,
    ];

    let mut cfg = de::DeConfig::default();
    cfg.max_iter = max_iter;
    if let Some(s) = seed {
        cfg.seed = s;
    }

    let points = &dataset.points;
    let mut scratch = dataset.scratch.borrow_mut();
    let (best, _best_energy, _iters) = de::differential_evolution(
        &bounds,
        |x| {
            let params = Params::from_slice(x);
            calc_loss(points, &params, &mut scratch).0
        },
        &cfg,
    );

    // Splice in the LS-optimal SM, matching the Python fit_orbit convention.
    let best_params = Params::from_slice(&best);
    let (_, sm) = calc_loss(points, &best_params, &mut scratch);
    let mut out = best;
    out[0] = sm;
    Ok(out)
}

#[cfg(feature = "python")]
#[pymodule]
fn binary_orbits_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_loss_py, m)?)?;
    m.add_function(wrap_pyfunction!(optimal_sm_py, m)?)?;
    m.add_function(wrap_pyfunction!(calc_positions_py, m)?)?;
    m.add_function(wrap_pyfunction!(fit_orbit_py, m)?)?;
    m.add_class::<Dataset>()?;
    m.add("NEWTON_ITERATIONS", NEWTON_ITERATIONS)?;
    Ok(())
}
