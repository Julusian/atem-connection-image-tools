#![feature(portable_simd)]

use constants::YuvConstants;
use napi::{Env, JsBuffer, JsUndefined};
use rgba_to_yuva422::rgb_to_yuv422;
use rgba_to_yuva422_simd::{rgb_to_yuv422_simd, YuvConstantsSimd};

mod constants;
mod rgba_to_yuva422;
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

  let constants = YuvConstants::create(kr, kb);

  let sample_count = pixel_count / 2;
  // for i in 0..sample_count {
  //   let offset = i * 8;

  //   let offset4 = offset + 4;
  //   let offset8 = offset + 8;
  //   rgb_to_yuv422(
  //     &constants,
  //     &input_vec[offset..offset4],
  //     &input_vec[offset4..offset8],
  //     &mut output_vec[offset..offset8],
  //   );
  // }

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

  // let row_batch_count = (height / 4) as usize;
  // for i in 0..row_batch_count {
  //   let offset_row1 = (i * 4 + 0) * row_batch_count;
  //   let offset_row2 = (i * 4 + 1) * row_batch_count;
  //   let offset_row3 = (i * 4 + 2) * row_batch_count;
  //   let offset_row4 = (i * 4 + 3) * row_batch_count;
  //   let offset_row_end = (i * 4 + 4) * row_batch_count;

  //   rgb_to_yuv422_row(
  //     &constants,
  //     &input_vec[offset_row1..offset_row2],
  //     &input_vec[offset_row2..offset_row3],
  //     &input_vec[offset_row3..offset_row4],
  //     &input_vec[offset_row4..offset_row_end],
  //     &mut output_vec[offset_row1..offset_row2],
  //     &mut output_vec[offset_row2..offset_row3],
  //     &mut output_vec[offset_row3..offset_row4],
  //     &mut output_vec[offset_row4..offset_row_end],
  //   );
  // }

  env.get_undefined()
}
