/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DisplayImage {
    #[serde(skip_serializing)]
    image: RgbaImage,
    width: u32,
    height: u32,
    size: usize,
    #[serde(serialize_with = "format_serde")]
    format: ImageFormat,
}

fn format_serde<S>(image_format: &ImageFormat, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{:?}", image_format).to_lowercase())
}

impl DisplayImage {
    pub fn new(img: RgbaImage, size: usize, format: ImageFormat) -> Self {
        let width = img.width();
        let height = img.height();
        let xstride = (width + 0b1111) & !0b1111;
        if xstride == width {
            return Self {
                width,
                height,
                size,
                format,
                image: img,
            };
        }

        let image = ImageBuffer::from_fn(xstride, height, |x, y| {
            if x < width {
                *img.get_pixel(x, y)
            } else {
                Rgba([0u8; 4])
            }
        });

        Self {
            width,
            height,
            size,
            format,
            image,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn len(&self) -> u32 {
        self.image.len() as u32
    }

    pub fn as_raw(&self) -> &[u8] {
        self.image.as_raw()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AspectMode {
    Fill,
    Fit,
}

#[derive(Debug, Copy, Clone)]
pub enum ContentMode {
    None,
    Aspect(AspectMode),
    ScaleToFill,
}

impl Serialize for ContentMode {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ContentMode::Aspect(mode) => s.serialize_str(&format!("Aspect{:?}", mode)),
            _ => s.serialize_str(&format!("{:?}", *self)),
        }
    }
}

#[derive(Debug)]
pub struct DisplayRect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

impl DisplayRect {
    pub fn new_with_mode(mode: ContentMode, viewport: (u32, u32), image: (u32, u32)) -> Self {
        let ((vw, vh), (w, h)) = (viewport, image);
        let viewport_aspect = vw as f32 / vh as f32;
        let image_aspect = w as f32 / h as f32;
        let ratio = image_aspect / viewport_aspect;
        let (vw, vh, w, h) = (vw as i16, vh as i16, w as i16, h as i16);

        match mode {
            ContentMode::None => Self {
                x: (vw - w) / 2,
                y: (vh - h) / 2,
                w: w,
                h: h,
            },
            ContentMode::ScaleToFill => Self {
                x: 0i16,
                y: 0i16,
                w: vw,
                h: vh,
            },
            ContentMode::Aspect(mode) => {
                let w = (vw as f32 * ratio) as i16;
                let h = (vh as f32 / ratio) as i16;
                let ratio = match mode {
                    AspectMode::Fit => ratio,
                    AspectMode::Fill => 1f32 / ratio,
                };
                if ratio == 1f32 {
                    Self {
                        x: 0i16,
                        y: 0i16,
                        w: vw,
                        h: vh,
                    }
                } else if ratio < 1f32 {
                    Self {
                        x: ((vw - w) / 2),
                        y: 0,
                        w: w,
                        h: vh,
                    }
                } else {
                    Self {
                        x: 0,
                        y: ((vh - h) / 2),
                        w: vw,
                        h: h,
                    }
                }
            }
        }
    }
}
