use image::{GenericImage, Rgba};
#[derive(Clone, Copy)]
pub struct SubImg {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl SubImg {
    pub fn draw(
        &self,
        img: &mut impl GenericImage<Pixel = Rgba<u8>>,
        x: u32,
        y: u32,
        color: Rgba<u8>,
    ) {
        /*
        assert!(x <= self.width);
        assert!(y <= self.height);
        assert!(self.x + self.width <= img.width());
        assert!(self.y + self.height <= img.height());
        */
        img.put_pixel(self.x + x, self.y + y, color);
    }

    pub fn rect(
        &self,
        img: &mut impl GenericImage<Pixel = Rgba<u8>>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
    ) {
        for i in x..x + width {
            for ii in y..y + height {
                self.draw(img, i, ii, color)
            }
        }
    }

    pub fn make_sub_img(&self, x: u32, y: u32, width: u32, hight: u32) -> Self {
        assert!(x + width <= self.width);
        assert!(y + hight <= self.height);
        SubImg {
            x: self.x + x,
            y: self.y + y,
            width,
            height: hight,
        }
    }
}
