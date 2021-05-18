/*
Copyright (c) 2021, Yuki MIZUNO
SPDX-License-Identifier: BSD-3-Clause
*/

pub mod vc {
    pub mod omx {
        pub fn init() {}
        pub fn deinit() {}

        pub fn get_display_size(_display_number: u16) -> (u32, u32) {
            (0, 0)
        }
    }

    pub mod tv {
        pub fn hdmi_power_on_preferred() {}
        pub fn power_off() {}
    }
}

pub mod pipeline {
    use crate::display::image::*;
    use crate::error::PipelineError;
    use gotham_derive::*;

    #[derive(Debug, Default, Copy, Clone)]
    pub struct Component {}

    #[derive(Debug, Default, Copy, Clone, StateData)]
    pub struct Pipeline {}

    impl Pipeline {
        pub fn new(_width: u32, _height: u32) -> Pipeline {
            Default::default()
        }

        pub fn init(&mut self) -> Result<(), PipelineError> {
            Ok(())
        }

        pub fn destroy(&mut self) -> Result<(), PipelineError> {
            Ok(())
        }

        pub fn render_image(
            &mut self,
            _image: &DisplayImage,
            _content_mode: ContentMode,
            _timeout: i32,
        ) -> Result<(), PipelineError> {
            Ok(())
        }
    }
}
