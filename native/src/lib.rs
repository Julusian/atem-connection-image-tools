#![feature(portable_simd)]

use napi::{Env, JsBuffer, JsUndefined};
use rgba_to_yuva422_simd::rgb_to_yuva422_simd;
use yuv_constants::YuvConstantsSimd;
use yuva422_to_rgba_simd::yuva422_to_rgb_simd;

mod rgba_to_yuva422_simd;
mod rle;
mod yuv_constants;
mod yuva422_to_rgba_simd;

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
  // @todo: MINT - 2018-5-24:
  // Create util functions that handle proper colour spaces in UHD.

  let input_vec = input.into_value()?;
  let mut output_vec = output.into_value()?;

  let pixel_count = (width * height) as usize;
  if width % 8 != 0 {
    env.throw_error("Width must be a multiple of 8", None)?;
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

  let sample_count = pixel_count / 2;

  let constants_simd = YuvConstantsSimd::create(kr, kb);

  let batch_count = sample_count / 4;
  for i in 0..batch_count {
    let offset_start = i * 32;
    let offset_end = offset_start + 32;

    rgb_to_yuva422_simd(
      &constants_simd,
      &input_vec[offset_start..offset_end],
      &mut output_vec[offset_start..offset_end],
    );
  }

  env.get_undefined()
}

/// Convert a RGBA buffer to ATEM YUV422 packing in the correct colorspace
///
/// This is performed synchronously
///
/// @param width - The width of the image
/// @param height - The height of the image
/// @param input - The input RGBA pixel data
/// @param output - The output YUVA422 pixel data
#[napi]
pub fn convert_yuva_422_to_rgba(
  env: Env,
  width: u32,
  height: u32,
  input: JsBuffer,
  output: JsBuffer,
) -> napi::Result<JsUndefined> {
  // @todo: MINT - 2018-5-24:
  // Create util functions that handle proper colour spaces in UHD.

  let input_vec = input.into_value()?;
  let mut output_vec = output.into_value()?;

  let pixel_count = (width * height) as usize;
  if width % 8 != 0 {
    env.throw_error("Width must be a multiple of 8", None)?;
    return env.get_undefined();
  }
  let byte_count = pixel_count * 4;
  // if input_vec.len() % 32 != 0 {
  //   env.throw_error("Input buffer has incorrect length", None)?;
  //   return env.get_undefined();
  // }
  if output_vec.len() != byte_count {
    env.throw_error("Output buffer has incorrect length", None)?;
    return env.get_undefined();
  }

  let [kr, kb] = if height >= 720 {
    [0.2126, 0.0722] // BT.709
  } else {
    [0.299, 0.114] // BT.601
  };

  let constants_simd = YuvConstantsSimd::create(kr, kb);

  let mut write_offset = 0;
  let mut decoder = rle::RLEDecoder::new(&input_vec);

  // let batch_count = sample_count / 4;
  while write_offset < output_vec.len() {
    let sample1 = decoder.read_sample();
    let sample2 = decoder.read_sample();
    let sample3 = decoder.read_sample();
    let sample4 = decoder.read_sample();

    // TODO - this could be handled better if needed, but that is difficult to do sanely
    if sample1.is_none() || sample2.is_none() || sample3.is_none() || sample4.is_none() {
      break; // Not enough data
    }

    let old_write_offset = write_offset;
    write_offset += 32;

    yuva422_to_rgb_simd(
      &constants_simd,
      &sample1.unwrap(),
      &sample2.unwrap(),
      &sample3.unwrap(),
      &sample4.unwrap(),
      &mut output_vec[old_write_offset..write_offset],
    );
  }

  // Throw if not enough data was decoded
  if write_offset < output_vec.len() {
    env.throw_error("Input buffer has less data than expected", None)?;
    return env.get_undefined();
  }

  env.get_undefined()
}
