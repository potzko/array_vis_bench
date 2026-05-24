//! Property: each algorithm produces the same `SortLog` trace on two
//! identical runs.
//!
//! Catches accidental nondeterminism (`HashMap` iteration order,
//! uninitialised aux memory, unseeded RNGs) that would otherwise only
//! show up as visual flicker in rendered GIFs. Algorithms whose
//! behaviour is *intentionally* random opt out via
//! `register_nondeterministic!`.

use std::io::Write;
use std::panic::AssertUnwindSafe;

use sort_logger::{SortLog, VisualizerLogger};

use crate::bench_registry::{
    is_nondeterministic, max_n_for_tests, primary_input, AlgorithmEntry, RunConfig, ALGORITHMS,
};

/// Default trace-comparison size. Per-algorithm clamped further by
/// `max_n_for_tests` so exponential-trace sorts (e.g.
/// `slow_sort_potzko`, cap=20) don't blow the heap at size=30.
const DEFAULT_SIZE: usize = 30;

#[test]
fn all_registered_algorithms_are_deterministic() {
    let total = ALGORITHMS.len();
    let mut checked = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut stderr = std::io::stderr();
    let suite_start = std::time::Instant::now();

    for (idx, entry) in ALGORITHMS.iter().enumerate() {
        if is_nondeterministic(entry.name) {
            skipped += 1;
            let _ = writeln!(
                stderr,
                "  [{}/{}] {} ... skipped (nondeterministic)",
                idx + 1,
                total,
                entry.name
            );
            continue;
        }

        let input_name = primary_input(entry.category);
        let size = max_n_for_tests(entry.name).map_or(DEFAULT_SIZE, |c| c.min(DEFAULT_SIZE));
        let det_config = RunConfig { size, seed: 42 };

        // Run once and return the *normalised* trace. Normalising inside
        // the closure means the raw trace is freed before we return —
        // important for exotic sorts whose `VisualizerLogger` trace can
        // be many GB even at small N. `normalize_trace` is bijective
        // length-wise, so the returned Vec's `len()` equals the raw
        // event count for diagnostics.
        let run_once = |entry: &AlgorithmEntry| -> Result<Vec<SortLog<usize>>, String> {
            let mut logger: VisualizerLogger<usize> = VisualizerLogger {
                type_ghost: std::marker::PhantomData,
                log: Vec::new(),
            };
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                (entry.run_with_input)(input_name, &det_config, &mut logger);
            }));
            match result {
                Ok(()) => Ok(normalize_trace(&logger.log)),
                Err(payload) => Err(format!("panic during run: {}", panic_payload_to_string(&payload))),
            }
        };

        let prefix = format!("[{}/{}] {}", idx + 1, total, entry.name);
        let _ = write!(stderr, "  {prefix} ... ");
        let _ = stderr.flush();

        let outcome = (|| {
            let na = run_once(entry)?;
            let nb = run_once(entry)?;
            if na == nb {
                Ok(na.len())
            } else {
                Err(format!(
                    "trace mismatch: {} vs {} events; first diff at index {}",
                    na.len(),
                    nb.len(),
                    first_diff_index(&na, &nb),
                ))
            }
        })();

        match outcome {
            Ok(n_events) => {
                let _ = writeln!(stderr, "ok ({} events)", n_events);
                checked += 1;
            }
            Err(msg) => {
                let _ = writeln!(stderr, "FAIL");
                let _ = writeln!(stderr, "    {msg}");
                failures.push((entry.name.to_string(), msg));
            }
        }
    }

    assert!(total > 0, "No algorithms registered in ALGORITHMS");

    let elapsed = suite_start.elapsed();
    let _ = writeln!(
        stderr,
        "\nSUMMARY: {} ok, {} failed, {} skipped of {} total ({:?})",
        checked,
        failures.len(),
        skipped,
        total,
        elapsed
    );
    if !failures.is_empty() {
        let _ = writeln!(stderr, "Failed:");
        for (name, _) in &failures {
            let _ = writeln!(stderr, "  ✘ {name}");
        }
        panic!(
            "{} of {} algorithms failed determinism check:\n\n{}",
            failures.len(),
            total - skipped,
            failures
                .into_iter()
                .map(|(name, m)| format!("{name}: {m}"))
                .collect::<Vec<_>>()
                .join("\n\n")
        );
    }
}

fn panic_payload_to_string(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// Replace pointer-based `name: usize` fields in a trace with stable IDs
/// assigned in first-seen order. Two runs of the same algorithm against
/// the same input emit the same *normalised* trace regardless of how the
/// OS happened to lay out aux-array allocations.
fn normalize_trace(trace: &[SortLog<usize>]) -> Vec<SortLog<usize>> {
    let mut map: std::collections::HashMap<usize, usize> = Default::default();
    let mut counter = 0usize;
    let mut assign = |n: usize| -> usize {
        *map.entry(n).or_insert_with(|| {
            let id = counter;
            counter += 1;
            id
        })
    };
    let mut out = Vec::with_capacity(trace.len());
    for e in trace {
        out.push(match e {
            SortLog::Swap { name, ind_a, ind_b } => SortLog::Swap {
                name: assign(*name),
                ind_a: *ind_a,
                ind_b: *ind_b,
            },
            SortLog::Mark(s) => SortLog::Mark(s.clone()),
            SortLog::CreateAuxArrT { name, length } => SortLog::CreateAuxArrT {
                name: assign(*name),
                length: *length,
            },
            SortLog::CreateAuxArr { name, length } => SortLog::CreateAuxArr {
                name: assign(*name),
                length: *length,
            },
            SortLog::FreeAuxArr { name } => SortLog::FreeAuxArr { name: assign(*name) },
            SortLog::CmpInArr { name, ind_a, ind_b, result } => SortLog::CmpInArr {
                name: assign(*name),
                ind_a: *ind_a,
                ind_b: *ind_b,
                result: *result,
            },
            SortLog::CmpData { name, ind, data, result } => SortLog::CmpData {
                name: assign(*name),
                ind: *ind,
                data: *data,
                result: *result,
            },
            SortLog::CmpDataU { name, ind, data, result } => SortLog::CmpDataU {
                name: assign(*name),
                ind: *ind,
                data: *data,
                result: *result,
            },
            SortLog::CmpAcrossArrs { name_a, ind_a, name_b, ind_b, result } => {
                SortLog::CmpAcrossArrs {
                    name_a: assign(*name_a),
                    ind_a: *ind_a,
                    name_b: assign(*name_b),
                    ind_b: *ind_b,
                    result: *result,
                }
            }
            SortLog::WriteInArr { name, ind_a, ind_b } => SortLog::WriteInArr {
                name: assign(*name),
                ind_a: *ind_a,
                ind_b: *ind_b,
            },
            SortLog::WriteData { name, ind, data } => SortLog::WriteData {
                name: assign(*name),
                ind: *ind,
                data: *data,
            },
            SortLog::WriteDataU { name, ind, data } => SortLog::WriteDataU {
                name: assign(*name),
                ind: *ind,
                data: *data,
            },
            SortLog::SetScale { name, max } => SortLog::SetScale {
                name: assign(*name),
                max: *max,
            },
            SortLog::SetScaleU { name, max } => SortLog::SetScaleU {
                name: assign(*name),
                max: *max,
            },
        });
    }
    out
}

fn first_diff_index(a: &[SortLog<usize>], b: &[SortLog<usize>]) -> usize {
    a.iter()
        .zip(b.iter())
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()))
}
