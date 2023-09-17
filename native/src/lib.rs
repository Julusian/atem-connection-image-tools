#![feature(portable_simd)]

use napi::{Env, JsBuffer, JsUndefined};
use rgba_to_yuva422_simd::{rgb_to_yuv422_simd, YuvConstantsSimd};

mod rgba_to_yuva422_simd;

#[macro_use]
extern crate napi_derive;

/// Convert a RGBA buffer to ATEM YUV422 packing in the correct colorspace
///
/// This is performed synchronously
///
/// @param width - The width of the image
/// @param height - The height of the image
/// @param input - The input RGBA pixel data
/// @param output - The output YUVA422 pixel data
#[napi]
pub fn convert_rgba_to_yuva_422(
  env: Env,
  width: u32,
  height: u32,
  input: JsBuffer,
  output: JsBuffer,
) -> napi::Result<JsUndefined> {
  // @todo: BALTE - 2018-5-24:
  // Create util functions that handle proper colour spaces in UHD.

  let input_vec = input.into_value()?;
  let mut output_vec = output.into_value()?;

  let pixel_count = (width * height) as usize;
  if width % 2 != 0 {
    env.throw_error("Width must be a multiple of 2", None)?;
    return env.get_undefined();
  }
  let byte_count = pixel_count * 4;
  if input_vec.len() != byte_count {
    env.throw_error("Input buffer has incorrect length", None)?;
    return env.get_undefined();
  }
  if output_vec.len() != byte_count {
    env.throw_error("Output buffer has incorrect length", None)?;
    return env.get_undefined();
  }

  let [kr, kb] = if height >= 720 {
    [0.2126, 0.0722] // BT.709
  } else {
    [0.299, 0.114] // BT.601
  };

  // // TODO _ HACK
  // if height % 4 != 0 {
  //   env.throw_error("Output buffer has incorrect length", None)?;
  //   return env.get_undefined();
  // }

  let sample_count = pixel_count / 2;

  let constants_simd = YuvConstantsSimd::create(kr, kb);

  let batch_count = sample_count / 4;
  for i in 0..batch_count {
    let offset_start = i * 32;
    let offset_end = offset_start + 32;

    rgb_to_yuv422_simd(
      &constants_simd,
      &input_vec[offset_start..offset_end],
      &mut output_vec[offset_start..offset_end],
    );
  }

  env.get_undefined()
}
