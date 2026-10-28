# `sort_vis`

The renderer. Takes the `SortLog<usize>` stream a `VisualizerLogger`
recorded and streams frames to `ffmpeg`, producing an MP4.

- `src/mp4_bar_visualiser.rs`: `Mp4Visualizer`, `Mp4Config` (resolution, frame rate, pacing, encoding, output path), and the common resolution and frame rate tables. A held buffer region is shaded blue and a held key region amber.
- `src/sub_image.rs`: frame layout helpers.

It is a separate crate so the `image` dependency and the ffmpeg process stay
out of every crate that only needs the logger.
