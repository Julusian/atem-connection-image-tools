//! The conversion loops, compiled for the baseline target and, on x86_64, also with AVX2 + FMA.
//! The best supported version is chosen at runtime.

use crate::rgba_to_yuva422_simd::rgb_to_yuva422_simd;
use crate::rle::RLEDecoder;
use crate::yuv_constants::YuvConstantsSimd;
use crate::yuva422_to_rgba_simd::yuva422_to_rgb_simd;

/// Convert RGBA to YUVA422, in batches of 8 pixels (32 bytes)
pub fn encode(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) {
  #[cfg(target_arch = "x86_64")]
  if has_avx2() {
    // Safety: the cpu supports the required features
    return unsafe { encode_avx2(constants, input, output) };
  }

  encode_impl(constants, input, output)
}

/// Decode (and un-RLE) YUVA422 to RGBA, returning the number of bytes written to the output
pub fn decode(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) -> usize {
  #[cfg(target_arch = "x86_64")]
  if has_avx2() {
    // Safety: the cpu supports the required features
    return unsafe { decode_avx2(constants, input, output) };
  }

  decode_impl(constants, input, output)
}

#[cfg(target_arch = "x86_64")]
fn has_avx2() -> bool {
  // The result is cached by std
  is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma")
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
fn encode_avx2(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) {
  encode_impl(constants, input, output)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
fn decode_avx2(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) -> usize {
  decode_impl(constants, input, output)
}

#[inline(always)]
fn encode_impl(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) {
  for (src, dst) in input.chunks_exact(32).zip(output.chunks_exact_mut(32)) {
    rgb_to_yuva422_simd(constants, src, dst);
  }
}

#[inline(always)]
fn decode_impl(constants: &YuvConstantsSimd, input: &[u8], output: &mut [u8]) -> usize {
  let mut decoder = RLEDecoder::new(input);
  let mut write_offset = 0;

  for dst in output.chunks_exact_mut(32) {
    let (Some(sample1), Some(sample2), Some(sample3), Some(sample4)) = (
      decoder.read_sample(),
      decoder.read_sample(),
      decoder.read_sample(),
      decoder.read_sample(),
    ) else {
      break; // Not enough data
    };

    yuva422_to_rgb_simd(constants, sample1, sample2, sample3, sample4, dst);
    write_offset += 32;
  }

  write_offset
}
