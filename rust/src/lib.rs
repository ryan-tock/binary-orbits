//! Rust port of the binary-orbits inner math.
//!
//! The Python reference (`server.py::calc_positions`, `calc_loss`) is the
//! source of truth for accuracy. Unit tests in `tests/parity.rs` lock us
//! to it within float tolerance; the criterion benches in `benches/` cover
//! speed. Tracing features live behind `trace-iter`, `trace-point`, and
//! `trace-hot` so they cost nothing in release builds.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

pub const NEWTON_ITERATIONS: usize = 6;

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
pub fn calc_positions(points: &[Point], params: &Params, out: &mut Vec<[f64; 2]>) {
    out.clear();
    out.reserve(points.len());

    let node_shifted = params.node - 3.0 * std::f64::consts::PI / 2.0;
    let node_cos = node_shifted.cos();
    let node_sin = node_shifted.sin();
    let inclined_angle = params.i.cos();
    let beta = params.e / (1.0 + (1.0 - params.e * params.e).sqrt());
    let two_pi_over_p = 2.0 * std::f64::consts::PI / params.p;

    for point in points {
        let mean_anomaly = params.m0 + two_pi_over_p * (point.t - 2000.0);

        let mut eccentric_anomaly = mean_anomaly;
        for _ in 0..NEWTON_ITERATIONS {
            let (sin_e, cos_e) = eccentric_anomaly.sin_cos();
            eccentric_anomaly += (mean_anomaly - eccentric_anomaly + params.e * sin_e)
                / (1.0 - params.e * cos_e);
        }

        let (sin_e, cos_e) = eccentric_anomaly.sin_cos();
        let true_anomaly = eccentric_anomaly
            + 2.0 * (beta * sin_e / (1.0 - beta * cos_e)).atan();

        let r = params.sm * (1.0 - params.e * cos_e);
        let (sin_w, cos_w) = (true_anomaly + params.periapsis).sin_cos();

        out.push([
            r * (cos_w * node_cos - inclined_angle * sin_w * node_sin),
            r * (inclined_angle * sin_w * node_cos + cos_w * node_sin),
        ]);
    }
}

/// Mirror of `calc_loss`. Returns both the loss and the least-squares
/// optimal semi-major axis so the caller can splice it back into the
/// parameter vector after DE finishes. `params.sm` is ignored on input
/// (matches the Python original, which overwrites it internally).
pub fn calc_loss(points: &[Point], params: &Params, scratch: &mut Vec<[f64; 2]>) -> (f64, f64) {
    // Compute positions with unit semi-major; the actual SM drops out of
    // the weighted least-squares ratio below.
    let unit_params = Params { sm: 1.0, ..*params };
    calc_positions(points, &unit_params, scratch);

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

    let mut error = 0.0;
    for (point, pred) in points.iter().zip(scratch.iter()) {
        let dx = point.x - sm * pred[0];
        let dy = point.y - sm * pred[1];
        error += (dx * dx + dy * dy) * point.weight;
    }

    (error, sm)
}

// --- PyO3 bindings ---

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

/// Python: `calc_loss(parameters: Sequence[float], data: list[dict]) -> float`
///
/// Returns the loss only — matches what scipy's differential_evolution wants
/// from its objective. Use `optimal_sm` separately to get the semi-major
/// axis after DE converges.
#[pyfunction]
#[pyo3(name = "calc_loss")]
fn calc_loss_py(parameters: Vec<f64>, data: Bound<'_, PyList>) -> PyResult<f64> {
    let points = points_from_pylist(&data)?;
    let params = Params::from_slice(&parameters);
    let mut scratch = Vec::with_capacity(points.len());
    let (loss, _) = calc_loss(&points, &params, &mut scratch);

    #[cfg(feature = "trace-iter")]
    eprintln!("calc_loss n={} loss={:.6e}", points.len(), loss);

    Ok(loss)
}

/// Python: `optimal_sm(parameters, data) -> float` — returns the LS-optimal
/// semi-major axis for a given parameter vector (ignores parameters[0]).
#[pyfunction]
#[pyo3(name = "optimal_sm")]
fn optimal_sm_py(parameters: Vec<f64>, data: Bound<'_, PyList>) -> PyResult<f64> {
    let points = points_from_pylist(&data)?;
    let params = Params::from_slice(&parameters);
    let mut scratch = Vec::with_capacity(points.len());
    let (_, sm) = calc_loss(&points, &params, &mut scratch);
    Ok(sm)
}

/// Python: `calc_positions(parameters, data) -> list[list[float]]`
///
/// Returned pairs are raw (sm=parameters[0] is applied), matching the
/// Python function's output shape.
#[pyfunction]
#[pyo3(name = "calc_positions")]
fn calc_positions_py(
    parameters: Vec<f64>,
    data: Bound<'_, PyList>,
) -> PyResult<Vec<[f64; 2]>> {
    let points = points_from_pylist(&data)?;
    let params = Params::from_slice(&parameters);
    let mut out = Vec::with_capacity(points.len());
    calc_positions(&points, &params, &mut out);
    Ok(out)
}

#[pymodule]
fn binary_orbits_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc_loss_py, m)?)?;
    m.add_function(wrap_pyfunction!(optimal_sm_py, m)?)?;
    m.add_function(wrap_pyfunction!(calc_positions_py, m)?)?;
    m.add("NEWTON_ITERATIONS", NEWTON_ITERATIONS)?;
    Ok(())
}
