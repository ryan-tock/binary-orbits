//! Accuracy tests: pin the Rust implementation to the Python reference
//! (server.py::calc_positions, calc_loss) via golden values captured with
//! `scripts/record_parity.py`. Every Rust optimization must keep these
//! passing within `TOL` of the recorded reference.

use approx::assert_relative_eq;
use binary_orbits_rs::{calc_loss, calc_positions, Params, Point};

const TOL: f64 = 1e-14;

fn points_a() -> Vec<Point> {
    vec![
        Point { t: 2000.0, x: 1.0, y: 0.0, weight: 1.0 },
        Point { t: 2001.0, x: 0.5, y: 0.8, weight: 1.0 },
        Point { t: 2002.0, x: -0.5, y: 0.8, weight: 1.0 },
        Point { t: 2003.0, x: -1.0, y: 0.0, weight: 1.0 },
    ]
}

fn points_b() -> Vec<Point> {
    vec![
        Point { t: 1998.0, x: 0.9, y: 0.1, weight: 2.0 },
        Point { t: 1999.0, x: 0.6, y: 0.6, weight: 1.5 },
        Point { t: 2000.5, x: 0.0, y: 0.95, weight: 1.0 },
        Point { t: 2002.25, x: -0.75, y: 0.55, weight: 0.8 },
        Point { t: 2004.0, x: -1.05, y: -0.2, weight: 1.2 },
        Point { t: 2006.5, x: -0.2, y: -0.9, weight: 1.0 },
        Point { t: 2008.0, x: 0.5, y: -0.7, weight: 0.9 },
        Point { t: 2010.0, x: 0.85, y: -0.3, weight: 1.0 },
    ]
}

#[test]
fn calc_positions_case1() {
    // sm=1.0, non-trivial everything
    let params = Params { sm: 1.0, e: 0.2, i: 0.5, node: 1.0, periapsis: 0.3, m0: 0.1, p: 6.0 };
    let expected: [[f64; 2]; 4] = [
        [-0.7727950631355025, 0.13041623005832229],
        [-0.22028371264877472, -0.81758487330832019],
        [0.70022516971655702, -0.87647405319533211],
        [1.1462027230332983, -0.28592346411487524],
    ];
    let mut out = Vec::new();
    calc_positions(&points_a(), &params, &mut out);
    assert_eq!(out.len(), expected.len());
    for (got, exp) in out.iter().zip(expected.iter()) {
        assert_relative_eq!(got[0], exp[0], max_relative = TOL, epsilon = TOL);
        assert_relative_eq!(got[1], exp[1], max_relative = TOL, epsilon = TOL);
    }
}

#[test]
fn calc_positions_case2_richer_dataset() {
    let params = Params { sm: 0.8, e: 0.4, i: 0.8, node: 2.5, periapsis: 1.2, m0: 2.0, p: 11.0 };
    let expected: [[f64; 2]; 8] = [
        [0.50903695118253656, 0.45079448177584713],
        [0.38433574040468182, 0.77306976438367725],
        [0.030803632728818166, 0.91315167023251342],
        [-0.39558626235737648, 0.7127974100934299],
        [-0.67484438945178338, 0.25165146604676925],
        [-0.37100843661252492, -0.46563925093504144],
        [0.37021307173382267, -0.071621064833772591],
        [0.38433574040468238, 0.77306976438367681],
    ];
    let mut out = Vec::new();
    calc_positions(&points_b(), &params, &mut out);
    assert_eq!(out.len(), expected.len());
    for (got, exp) in out.iter().zip(expected.iter()) {
        assert_relative_eq!(got[0], exp[0], max_relative = TOL, epsilon = TOL);
        assert_relative_eq!(got[1], exp[1], max_relative = TOL, epsilon = TOL);
    }
}

#[test]
fn calc_positions_case3_high_sm() {
    let params = Params { sm: 1.5, e: 0.05, i: 0.39, node: 1.57, periapsis: 1.5, m0: 1.5, p: 6.0 };
    let expected: [[f64; 2]; 4] = [
        [1.4970988863859531, -0.05878473117101058],
        [0.89900528382766798, 1.1827383118458024],
        [-0.52274620811919781, 1.3674615765414084],
        [-1.4652279945955251, 0.3339316175604225],
    ];
    let mut out = Vec::new();
    calc_positions(&points_a(), &params, &mut out);
    assert_eq!(out.len(), expected.len());
    for (got, exp) in out.iter().zip(expected.iter()) {
        assert_relative_eq!(got[0], exp[0], max_relative = TOL, epsilon = TOL);
        assert_relative_eq!(got[1], exp[1], max_relative = TOL, epsilon = TOL);
    }
}

#[test]
fn calc_loss_case1_sm_clamped_to_zero() {
    // LS estimate goes negative; calc_loss clamps to 0 and returns error
    // with that clamped value (matches the Python reference).
    let params = Params { sm: 1.0, e: 0.2, i: 0.5, node: 1.0, periapsis: 0.3, m0: 0.1, p: 6.0 };
    let mut scratch = Vec::new();
    let (loss, sm) = calc_loss(&points_a(), &params, &mut scratch);
    assert_relative_eq!(loss, 3.7800000000000002, max_relative = TOL);
    assert_eq!(sm, 0.0);
}

#[test]
fn calc_loss_case2_fits_with_positive_sm() {
    let params = Params { sm: 0.8, e: 0.4, i: 0.8, node: 2.5, periapsis: 1.2, m0: 2.0, p: 11.0 };
    let mut scratch = Vec::new();
    let (loss, sm) = calc_loss(&points_b(), &params, &mut scratch);
    assert_relative_eq!(loss, 3.1531067340209256, max_relative = TOL);
    assert_relative_eq!(sm, 0.76909319860473668, max_relative = TOL);
}

#[test]
fn calc_loss_case3_near_optimal() {
    let params = Params { sm: 1.5, e: 0.05, i: 0.39, node: 1.57, periapsis: 1.5, m0: 1.5, p: 6.0 };
    let mut scratch = Vec::new();
    let (loss, sm) = calc_loss(&points_a(), &params, &mut scratch);
    assert_relative_eq!(loss, 0.09302179303081877, max_relative = TOL);
    assert_relative_eq!(sm, 0.96798816353676276, max_relative = TOL);
}

#[test]
fn calc_loss_scratch_is_reusable() {
    let params = Params { sm: 0.8, e: 0.4, i: 0.8, node: 2.5, periapsis: 1.2, m0: 2.0, p: 11.0 };
    let mut scratch = Vec::new();
    let (loss1, _) = calc_loss(&points_b(), &params, &mut scratch);
    let (loss2, _) = calc_loss(&points_b(), &params, &mut scratch);
    assert_relative_eq!(loss1, loss2, max_relative = TOL);
}

#[test]
fn zero_params_are_finite() {
    // DE can land on the edges of bounds; make sure we don't NaN.
    let params = Params { sm: 0.0, e: 0.0, i: 0.0, node: 0.0, periapsis: 0.0, m0: 0.0, p: 6.0 };
    let mut scratch = Vec::new();
    let (loss, _) = calc_loss(&points_a(), &params, &mut scratch);
    assert!(loss.is_finite());
}
