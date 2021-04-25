use image::{ImageBuffer, Rgba, RgbaImage};

pub fn align_image(img: RgbaImage) -> RgbaImage {
    let xstride = (img.width() + 0b1111) & !0b1111;
    if xstride == img.width() {
        return img;
    }

    ImageBuffer::from_fn(xstride, img.height(), |x, y| {
        if x < img.width() {
            *img.get_pixel(x, y)
        } else {
            Rgba([0u8; 4])
        }
    })
}
