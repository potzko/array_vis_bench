//! Flat RGB24 framebuffer + a sub-rectangle view into it.
//!
//! This replaces the previous `image::ImageBuffer` + `GenericImage` plumbing
//! so the hot path goes straight to `Vec<u8>` index math (`memcpy`-like writes,
//! no virtual dispatch, no per-pixel `put_pixel`).

const RGB_BYTES: usize = 3;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width as usize) * (height as usize) * RGB_BYTES],
        }
    }

    /// Fill `[x, x+w) × [y, y+h)` with a solid RGB color. Out-of-bounds is clipped.
    pub fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 3]) {
        if w == 0 || h == 0 || x >= self.width || y >= self.height {
            return;
        }
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        let row_pixels = (x_end - x) as usize;
        let row_bytes = row_pixels * RGB_BYTES;
        let stride = self.width as usize * RGB_BYTES;
        let first_row_off = y as usize * stride + x as usize * RGB_BYTES;

        // Fill the first row pixel-by-pixel; the compiler vectorises this.
        {
            let row_slice = &mut self.data[first_row_off..first_row_off + row_bytes];
            for px in row_slice.chunks_exact_mut(RGB_BYTES) {
                px.copy_from_slice(&color);
            }
        }
        // Memcpy that row down for every subsequent row of the rect.
        for yy in (y + 1)..y_end {
            let dst = yy as usize * stride + x as usize * RGB_BYTES;
            self.data
                .copy_within(first_row_off..first_row_off + row_bytes, dst);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SubImg {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SubImg {
    pub fn rect(
        &self,
        fb: &mut Framebuffer,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: [u8; 3],
    ) {
        fb.rect(self.x + x, self.y + y, width, height, color);
    }

    pub fn make_sub_img(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        assert!(x + width <= self.width);
        assert!(y + height <= self.height);
        SubImg {
            x: self.x + x,
            y: self.y + y,
            width,
            height,
        }
    }
}
