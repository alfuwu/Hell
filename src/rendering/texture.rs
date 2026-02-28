use std::sync::Arc;
use vulkano::image::view::ImageView;

#[derive(Clone, PartialEq)]
pub enum SampleType {
    POINT,
    LINEAR
}

#[derive(Clone, PartialEq)]
pub struct Texture {
    pub texture: Arc<ImageView>,
    pub sample_type: SampleType,
}
impl Texture {
    pub const fn new(texture: Arc<ImageView>, sample_type: SampleType) -> Self {
        Self { texture, sample_type }
    }

    pub const fn point(texture: Arc<ImageView>) -> Self {
        Self { texture, sample_type: SampleType::POINT }
    }

    pub const fn linear(texture: Arc<ImageView>) -> Self {
        Self { texture, sample_type: SampleType::LINEAR }
    }
}