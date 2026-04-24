#!/usr/bin/env python3
"""Integration benchmark: measure fit_orbit wall time with Python vs Rust.

We don't just care about calc_loss microbenches — scipy's differential_
evolution calls the objective thousands of times, so the per-call FFI
overhead of calling into Rust can eat the gains. This harness runs the
full DE with each implementation and reports the speedup.
"""
from __future__ import annotations

import argparse
import copy
import json
import math
import statistics
import sys
import time
from pathlib import Path

# Import the Python reference.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import server as py_server  # noqa: E402

try:
    import binary_orbits_rs as rs  # type: ignore
except ImportError:
    rs = None
    print("warning: binary_orbits_rs not importable; run `maturin develop --release`")

from scipy.optimize import differential_evolution  # noqa: E402


def fit_python(data, period_bound):
    bounds = [
        (0, 0), (0, 0.95), (0, math.pi),
        (0, 2 * math.pi), (0, 2 * math.pi), (0, 2 * math.pi),
        (period_bound[0], period_bound[1]),
    ]
    res = differential_evolution(py_server.calc_loss, bounds, args=(data,), seed=42)
    params = res.x.tolist()
    py_server.calc_loss(params, data)
    return params


def fit_rust(data, period_bound):
    """scipy DE + Rust loss."""
    assert rs is not None
    bounds = [
        (0, 0), (0, 0.95), (0, math.pi),
        (0, 2 * math.pi), (0, 2 * math.pi), (0, 2 * math.pi),
        (period_bound[0], period_bound[1]),
    ]

    # Build the Dataset once so DE doesn't pay the dict→Vec conversion
    # on every objective call.
    ds = rs.Dataset(data)

    def loss(x):
        return rs.calc_loss(x.tolist(), ds)

    res = differential_evolution(loss, bounds, args=(), seed=42)
    params = res.x.tolist()
    params[0] = rs.optimal_sm(params, ds)
    return params


def fit_pure_rust(data, period_bound):
    """Rust DE + BFGS refine + Rust loss."""
    assert rs is not None
    ds = rs.Dataset(data)
    return rs.fit_orbit(ds, period_bound, seed=42, refine=True)


def fit_rust_de_no_refine(data, period_bound):
    """Rust DE only — no polish, to show the raw DE-vs-DE speed baseline."""
    assert rs is not None
    ds = rs.Dataset(data)
    return rs.fit_orbit(ds, period_bound, seed=42, refine=False)


def fit_rust_sgd(data, period_bound):
    """Multistart noisy-GD + BFGS refine (pure Rust)."""
    assert rs is not None
    ds = rs.Dataset(data)
    return rs.fit_orbit_sgd(ds, period_bound, seed=42, refine=True)


def timed(fn, reps):
    times = []
    result = None
    for _ in range(reps):
        t0 = time.perf_counter()
        result = fn()
        times.append(time.perf_counter() - t0)
    return times, result


def load_dataset(path: Path, orbit_key: str | None):
    payload = json.loads(path.read_text())
    if orbit_key is None:
        orbit_key = next(iter(payload))
    return payload[orbit_key]["data"], orbit_key


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default="sample-data.json")
    ap.add_argument("--orbit", default=None, help="Which orbit from the JSON to fit (default: first)")
    ap.add_argument("--period-low", type=float, default=2.0)
    ap.add_argument("--period-high", type=float, default=40.0)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--which",
                    choices=["python", "rust", "pure-rust", "de-only", "sgd", "all"],
                    default="all")
    args = ap.parse_args()

    data, orbit_key = load_dataset(Path(args.data), args.orbit)
    print(f"dataset: {args.data}  orbit={orbit_key!r}  n={len(data)}  reps={args.reps}")
    print(f"period bounds: [{args.period_low}, {args.period_high}]")
    print()

    results = {}

    def run(name, label, fn):
        print(f"[{name}] {label}")
        times, params = timed(lambda: fn(copy.deepcopy(data), (args.period_low, args.period_high)), args.reps)
        results[name] = (times, params)
        print(f"  times (s): {['%.3f' % t for t in times]}  median={statistics.median(times):.3f}s")
        print(f"  params: {[round(p, 4) for p in params]}")
        print()

    if args.which in ("python", "all"):
        run("python", "scipy DE + Python loss", fit_python)

    if args.which in ("rust", "all") and rs is not None:
        run("rust", "scipy DE + Rust loss", fit_rust)

    if args.which in ("de-only", "all") and rs is not None:
        run("de-only", "Rust DE only (no polish)", fit_rust_de_no_refine)

    if args.which in ("pure-rust", "all") and rs is not None:
        run("pure-rust", "Rust DE + BFGS refine (Rust polish)", fit_pure_rust)

    if args.which in ("sgd", "all") and rs is not None:
        run("sgd", "Rust multistart SGD + BFGS refine", fit_rust_sgd)

    if "python" in results:
        py_med = statistics.median(results["python"][0])
        py_loss = py_server.calc_loss(results["python"][1][:], data)
        print(f"{'python':<12}: {py_med*1000:7.0f} ms  loss={py_loss:.6g}")
        for name in ("rust", "de-only", "pure-rust", "sgd"):
            if name in results:
                med = statistics.median(results[name][0])
                loss = py_server.calc_loss(results[name][1][:], data)
                print(f"{name:<12}: {med*1000:7.0f} ms  loss={loss:.6g}  ({py_med/med:.1f}× faster than python)")


if __name__ == "__main__":
    main()
