use crate::bench_registry::{AlgorithmEntry, RunConfig, ALGORITHMS};
use sort_logger::{SortLogger, VisualizerLogger};
use sort_vis::{Mp4Config, Mp4Visualizer, Visualizer};

/// Find an algorithm by exact name. Returns `None` if no entry matches.
pub fn find(name: &str) -> Option<&'static AlgorithmEntry> {
    ALGORITHMS.iter().find(|e| e.name == name)
}

/// Run the named algorithm against the named input and render the
/// resulting log to an MP4. The input must be registered in the input
/// registry matching the algorithm's category; the algorithm emits all
/// events (CreateArr + initial Writes + sort operations) on `logger`,
/// so the log alone describes the full visualisation.
pub fn visualise(
    algorithm_name: &str,
    input_name: &str,
    config: &RunConfig,
    logger: &mut VisualizerLogger<usize>,
    mp4_config: Mp4Config,
) {
    let entry = find(algorithm_name).unwrap_or_else(|| {
        panic!("algorithm '{}' not in ALGORITHMS registry", algorithm_name)
    });
    (entry.run_with_input)(input_name, config, logger as &mut dyn SortLogger<usize>);

    let mut viz = Mp4Visualizer::new(mp4_config);
    viz.render(&logger.log);
}
