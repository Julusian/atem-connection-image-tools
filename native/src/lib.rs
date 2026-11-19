#![feature(portable_simd)]

use base64::Engine;
use napi::bindgen_prelude::{AsyncTask, Buffer};
use napi::{Env, Task};
use yuv_constants::YuvConstantsSimd;

mod dispatch;
mod rgba_to_yuva422_simd;
mod rle;
mod yuv_constants;
mod yuva422_to_rgba_simd;

#[macro_use]
extern crate napi_derive;

fn invalid_arg(msg: &str) -> napi::Error {
  napi::Error::new(napi::Status::InvalidArg, msg)
}

/// Check the dimensions, and return the number of bytes of RGBA data for the image
fn check_dimensions(width: u32, height: u32) -> napi::Result<usize> {
  // @todo: MINT - 2018-5-24:
  // Create util functions that handle proper colour spaces in UHD.
  if width % 8 != 0 {
    return Err(invalid_arg("Width must be a multiple of 8"));
  }
  Ok(width as usize * height as usize * 4)
}

fn constants_for_height(height: u32) -> YuvConstantsSimd {
  if height >= 720 {
    YuvConstantsSimd::create(0.2126, 0.0722) // BT.709
  } else {
    YuvConstantsSimd::create(0.299, 0.114) // BT.601
  }
}

fn check_encode_args(width: u32, height: u32, input: &[u8], output: &[u8]) -> napi::Result<()> {
  let byte_count = check_dimensions(width, height)?;
  if input.len() != byte_count {
    return Err(invalid_arg("Input buffer has incorrect length"));
  }
  if output.len() != byte_count {
    return Err(invalid_arg("Output buffer has incorrect length"));
  }
  Ok(())
}

fn check_decode_args(width: u32, height: u32, output: &[u8]) -> napi::Result<()> {
  // RLE means the input data could have any length, so that gets checked while decoding
  let byte_count = check_dimensions(width, height)?;
  if output.len() != byte_count {
    return Err(invalid_arg("Output buffer has incorrect length"));
  }
  Ok(())
}

fn encode(height: u32, input: &[u8], output: &mut [u8]) {
  dispatch::encode(&constants_for_height(height), input, output);
}

fn decode(height: u32, input: &[u8], output: &mut [u8]) -> napi::Result<()> {
  let written = dispatch::decode(&constants_for_height(height), input, output);
  if written < output.len() {
    return Err(invalid_arg("Input buffer has less data than expected"));
  }
  Ok(())
}

fn hash(data: &[u8]) -> String {
  let hash = xxhash_rust::xxh3::xxh3_128(data);
  base64::engine::general_purpose::STANDARD.encode(hash.to_be_bytes())
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
pub fn convert_rgba_to_yuva_422(
  width: u32,
  height: u32,
  input: Buffer,
  mut output: Buffer,
) -> napi::Result<()> {
  check_encode_args(width, height, &input, &output)?;
  encode(height, &input, &mut output);
  Ok(())
}

/// Convert an ATEM YUVA422 buffer (optionally RLE encoded) to RGBA from the correct colorspace
///
/// This is performed synchronously
///
/// @param width - The width of the image
/// @param height - The height of the image
/// @param input - The input YUVA422 pixel data
/// @param output - The output RGBA pixel data
#[napi]
pub fn convert_yuva_422_to_rgba(
  width: u32,
  height: u32,
  input: Buffer,
  mut output: Buffer,
) -> napi::Result<()> {
  check_decode_args(width, height, &output)?;
  decode(height, &input, &mut output)
}

/// Generate a hash of a buffer, to identify the content of an image. Returned as a base64 string of 16 bytes
#[napi]
pub fn hash_buffer(data: Buffer) -> String {
  hash(&data)
}

pub struct EncodeTask {
  height: u32,
  input: Buffer,
  output: Buffer,
}
impl Task for EncodeTask {
  type Output = String;
  type JsValue = String;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    encode(self.height, &self.input, &mut self.output);
    Ok(hash(&self.output))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(output)
  }
}

/// Convert a RGBA buffer to ATEM YUV422 packing in the correct colorspace, and generate a hash of the output
///
/// This is performed on the libuv threadpool. The buffers must not be modified until the promise has settled
///
/// @param width - The width of the image
/// @param height - The height of the image
/// @param input - The input RGBA pixel data
/// @param output - The output YUVA422 pixel data
/// @returns The hash of the output data
#[napi(ts_return_type = "Promise<string>")]
pub fn convert_rgba_to_yuva_422_async(
  width: u32,
  height: u32,
  input: Buffer,
  output: Buffer,
) -> napi::Result<AsyncTask<EncodeTask>> {
  check_encode_args(width, height, &input, &output)?;
  Ok(AsyncTask::new(EncodeTask {
    height,
    input,
    output,
  }))
}

pub struct DecodeTask {
  height: u32,
  input: Buffer,
  output: Buffer,
}
impl Task for DecodeTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> napi::Result<Self::Output> {
    decode(self.height, &self.input, &mut self.output)
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(())
  }
}

/// Convert an ATEM YUVA422 buffer (optionally RLE encoded) to RGBA from the correct colorspace
///
/// This is performed on the libuv threadpool. The buffers must not be modified until the promise has settled
///
/// @param width - The width of the image
/// @param height - The height of the image
/// @param input - The input YUVA422 pixel data
/// @param output - The output RGBA pixel data
#[napi(ts_return_type = "Promise<void>")]
pub fn convert_yuva_422_to_rgba_async(
  width: u32,
  height: u32,
  input: Buffer,
  output: Buffer,
) -> napi::Result<AsyncTask<DecodeTask>> {
  check_decode_args(width, height, &output)?;
  Ok(AsyncTask::new(DecodeTask {
    height,
    input,
    output,
  }))
}
