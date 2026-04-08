# visualise

Bridge between the interactive binary and the GIF renderer. This module runs a sort with a `VisualizerLogger`, captures the full operation log, and hands it to the `sort_vis` crate for rendering.

## How it works

1. Clone the original array (needed as the "before" state for the renderer).
2. Run the selected sort via `sorts::fn_sort`, passing a `VisualizerLogger` that records every comparison, swap, and write.
3. Call `sort_vis::render_gif` with the original array, the array's base address (used to identify which logged writes target the main array vs. auxiliary buffers), and the collected log.
4. The `sort_vis` crate replays the log frame by frame and writes an animated GIF to disk.

## Why it's a thin wrapper

All rendering logic lives in the `sort_vis` crate. This module exists so the main binary doesn't need to know about GIF encoding — it just calls `visualise_sort` and gets an output file.
