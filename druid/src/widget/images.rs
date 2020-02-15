// Copyright 2020 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An Image widget.
//! Please consider using SVG and the SVG wideget as it scales much better.

use std::error::Error;
use std::marker::PhantomData;
use std::str::FromStr;

use image;

use crate::{
    Affine, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget,
};

use crate::piet::{ImageFormat, InterpolationMode};

#[derive(PartialEq)]
pub enum FillStrat {
    Contain,
    Cover,
    Fill,
    FitHeight,
    FitWidth,
    None,
    ScaleDown,
}

impl Default for FillStrat {
    fn default() -> Self {
        FillStrat::Fill
    }
}

fn get_scale_offset(parent: Size, fit_box: Size, fit_type: &FillStrat) -> (Point, Point) {
    let scalex = parent.width / fit_box.width;
    let scaley = parent.height / fit_box.height;

    let scale: Point = match fit_type {
        FillStrat::Contain => {
            let scale = scalex.min(scaley);
            Point { x: scale, y: scale }
        }
        FillStrat::Cover => {
            let scale = scalex.max(scaley);
            Point { x: scale, y: scale }
        }
        FillStrat::Fill => Point {
            x: scalex,
            y: scaley,
        },
        FillStrat::FitHeight => Point {
            x: scaley,
            y: scaley,
        },
        FillStrat::FitWidth => Point {
            x: scalex,
            y: scalex,
        },
        FillStrat::ScaleDown => {
            let scale = scalex.min(scaley).min(1.0);
            Point { x: scale, y: scale }
        }
        FillStrat::None => Point { x: 1.0, y: 1.0 },
    };

    let origin_x = (parent.width - (fit_box.width * scale.x)) / 2.0;
    let origin_y = (parent.height - (fit_box.height * scale.y)) / 2.0;
    let origin = Point::new(origin_x, origin_y);

    (scale, origin)
}

/// A widget that renders a Images
pub struct Image<T> {
    image_data: ImageData,
    phantom: PhantomData<T>,
    fill: FillStrat,
}

impl<T: Data> Image<T> {
    /// Create an Image-drawing widget from ImageData.
    ///
    /// The Image will scale to fit its box constraints.
    pub fn new(image_data: ImageData) -> Self {
        Image {
            image_data,
            phantom: Default::default(),
            fill: FillStrat::default(),
        }
    }

    fn get_size(&self) -> Size {
        Size::new(
            self.image_data.x_pixels as f64,
            self.image_data.y_pixels as f64,
        )
    }

    pub fn set_fill(&mut self, newfil: FillStrat) {
        self.fill = newfil;
    }
}

impl<T: Data> Widget<T> for Image<T> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &T,
        _env: &Env,
    ) -> Size {
        bc.debug_check("Image");

        if bc.is_width_bounded() {
            bc.max()
        } else {
            bc.constrain(self.get_size())
        }
    }
    fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &T, _env: &Env) {
        let bob = get_scale_offset(paint_ctx.size(), self.get_size(), &self.fill);

        // The ImageData's to_piet function does not clip to the image's size
        // CairoRenderContext is very like druids but with some extra goodies like clip
        if self.fill == FillStrat::Contain {
        } else {
            let clip_rect = Rect::ZERO.with_size(paint_ctx.size());
            paint_ctx.clip(clip_rect);
        }
        self.image_data.to_piet(bob.0.x, bob.1, paint_ctx);
    }
}

/// Stored Image data.
/// Implements `FromStr` and can be converted to piet draw instructions.
#[derive(Clone)]
pub struct ImageData {
    pixels: Vec<u8>,
    x_pixels: u32,
    y_pixels: u32,
}

impl ImageData {
    /// Create an empty Image
    pub fn empty() -> Self {
        ImageData {
            pixels: [].to_vec(),
            x_pixels: 0,
            y_pixels: 0,
        }
    }

    pub fn from_data(raw_image: &Vec<u8>) -> Result<Self, dyn Error> {
        let dec = image::load_from_memory(&raw_image[..]).unwrap().to_rgb();

        let sizeofimage = dec.dimensions();
        Ok(ImageData {
            pixels: dec.to_vec(),
            x_pixels: sizeofimage.0,
            y_pixels: sizeofimage.1,
        })
    }

    /// Convert ImageData into Piet draw instructions
    pub fn to_piet(&self, scale: f64, offset: Point, paint_ctx: &mut PaintCtx) {
        let offset_matrix = Affine::new([scale, 0., 0., scale, offset.x, offset.y]);

        paint_ctx
            .with_save(|ctx| {
                ctx.transform(offset_matrix);

                let im = ctx
                    .make_image(
                        self.x_pixels as usize,
                        self.y_pixels as usize,
                        &self.pixels,
                        ImageFormat::Rgb,
                    )
                    .unwrap();
                let rec = Rect::from_origin_size(
                    (0.0, 0.0),
                    (self.x_pixels as f64, self.y_pixels as f64),
                );
                ctx.draw_image(&im, rec, InterpolationMode::Bilinear);

                Ok(())
            })
            .unwrap();
    }
}

impl Default for ImageData {
    fn default() -> Self {
        ImageData::empty()
    }
}

impl FromStr for ImageData {
    type Err = Box<dyn Error>;

    fn from_str(image_str: &str) -> Result<Self, Self::Err> {
        let image_data = image::open(image_str).unwrap().to_rgb();
        // catch unrap

        let sizeofimage = image_data.dimensions();
        Ok(ImageData {
            pixels: image_data.to_vec(),
            x_pixels: sizeofimage.0,
            y_pixels: sizeofimage.1,
        })
    }
}
