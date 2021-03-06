/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/

use crate::display::image::*;

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
