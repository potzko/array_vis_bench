# sort_vis

GIF renderer for sort visualisations. Takes the `Vec<SortLog<usize>>` produced by a `VisualizerLogger` and turns it into an animated GIF on disk.

## What it contains

- **`render_gif(logs, arr, path, ...)`** — the single public entry point. Replays the log against the array state frame by frame and encodes each frame as a paletted image.
- **`img_tmp`** (internal) — frame generation: maps array values to pixel colours, draws comparison/swap highlights, composes frames.
- **`sub_image`** (internal) — sub-image slicing helpers used during frame layout.

## Why it's a separate crate

The `image` crate (and GIF encoding) is a heavy dependency that nothing except the visualiser binary needs. Isolating it here means the sort implementations and the benchmark binary compile without it. The root crate only pulls in `sort_vis` when building the visualiser.
