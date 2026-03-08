pub mod comb_sort;
pub mod combinations;
pub mod sequences;

use crate::traits::log_traits::SortLogger;
use sequences::COMB_SEQUENCES;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");

    for entry in COMB_SEQUENCES {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }

    // Default: classic (1.3) shrink factor
    let gaps = {
        let mut g = arr.len();
        let mut gs = Vec::new();
        while g > 1 {
            g = (g * 10 / 13).max(1);
            gs.push(g);
        }
        gs
    };
    comb_sort::CombSort::sort_with_gaps(arr, logger, gaps);
    vec![format!("name: {}", name)]
}

pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in COMB_SEQUENCES {
        if entry.name == name {
            return Some(vec!["comb_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
