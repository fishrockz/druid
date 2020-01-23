// Copyright 2019 The xi-editor Authors.
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
//! Please consider using Image and the Image wideget as it scales much better.

use std::error::Error;
use std::marker::PhantomData;
use std::str::FromStr;

use crate::{
    Affine, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, PaintCtx,
    Point, Rect, RenderContext, Size, UpdateCtx, Widget,
};

use crate::piet::{ImageFormat, InterpolationMode};

use image;

/// A widget that renders a Images
pub struct Image<T> {
    image_data: ImageData,
    phantom: PhantomData<T>,
}

impl<T: Data> Image<T> {
    /// Create an Image-drawing widget from ImageData.
    ///
    /// The Image will scale to fit its box constraints.
    pub fn new(image_data: ImageData) -> impl Widget<T> {
        Image {
            image_data,
            phantom: Default::default(),
        }
    }

    /// Measure the Image's size
    #[allow(clippy::needless_return)]
    fn get_size(&self) -> Size {
        // Fix me
        Size::new(self.image_data.x_pixels as f64, self.image_data.y_pixels as f64)
    }
}

impl<T: Data> Widget<T> for Image<T> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&T>, _data: &T, _env: &Env) {}

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
        //TODO: options for aspect ratio or scaling based on height
        let scalex = paint_ctx.size().width / self.get_size().width;
        let scaley = paint_ctx.size().height / self.get_size().height;
        let scale = scalex.min(scaley);

        let origin_x = (paint_ctx.size().width - (self.get_size().width * scale)) / 2.0;
        let origin_y = (paint_ctx.size().height - (self.get_size().height * scale)) / 2.0;
        let origin = Point::new(origin_x, origin_y);

        let clip_rect = Rect::ZERO.with_size(paint_ctx.size());

        // The ImageData's to_piet function dose not clip to the image's size
        // CairoRenderContext is very like druids but with some extra goodies like clip
        //paint_ctx.clip(clip_rect);
        self.image_data.to_piet(scale, origin, paint_ctx);
    }
}

/// Stored Image data.
/// Implements `FromStr` and can be converted to piet draw instructions.
#[derive(Clone)]
pub struct ImageData {
    pixels: Vec<u8>,
    x_pixels: i32,
    y_pixels: i32,
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



    pub fn from_data(rawImage: Vec<u8>) -> Self{
        let dec = image::load_from_memory(&rawImage[..])
            .unwrap()
            .to_rgb();

        let sizeofimage = dec.dimensions();
        let correct_bytes = dec.to_vec();
        ImageData{pixels:  dec.to_vec(), x_pixels: sizeofimage.0 as i32, y_pixels: sizeofimage.1 as i32}
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
                    (
                        self.x_pixels as f64,
                        self.y_pixels as f64,
                    ),
                );
                ctx.draw_image(&im, rec, InterpolationMode::Bilinear);

                Ok(())
            });

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
        //pub fn from_path(path: ) -> Self {
        let image_data = image::open(image_str).unwrap().to_rgb();
        // catch unrap

        let sizeofimage = image_data.dimensions();
        let correct_bytes = image_data.to_vec();
        Ok(ImageData{pixels:  image_data.to_vec(), x_pixels: sizeofimage.0 as i32, y_pixels: sizeofimage.1 as i32})
    }    
}


