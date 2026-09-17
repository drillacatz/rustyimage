pub mod apng_codec;
pub mod gif_codec;
pub mod static_codec;
pub mod svg_codec;

pub use apng_codec::{decode_apng, encode_apng};
pub use gif_codec::{decode_gif, encode_gif};
pub use static_codec::{
    export_animated_file, export_single_frame, load_any_image, load_multiple_images,
};

