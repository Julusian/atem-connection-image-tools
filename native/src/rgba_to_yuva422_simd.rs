use crate::yuv_constants::YuvConstantsSimd;
use std::simd::{Simd, SimdFloat, SimdUint};

#[inline(always)]
pub fn rgb_to_yuva422_simd(constants: &YuvConstantsSimd, input: &[u8], target: &mut [u8]) {
  let vec_r1 = Simd::gather_or_default(input, constants.gather_idx).cast::<f32>();
  let vec_g1 = Simd::gather_or_default(&input[1..], constants.gather_idx).cast::<f32>();
  let vec_b1 = Simd::gather_or_default(&input[2..], constants.gather_idx).cast::<f32>();
  let vec_a1 = Simd::gather_or_default(&input[3..], constants.gather_idx).cast::<f32>();

  let vec_r2 = Simd::gather_or_default(&input[4..], constants.gather_idx).cast::<f32>();
  let vec_g2 = Simd::gather_or_default(&input[5..], constants.gather_idx).cast::<f32>();
  let vec_b2 = Simd::gather_or_default(&input[6..], constants.gather_idx).cast::<f32>();
  let vec_a2 = Simd::gather_or_default(&input[7..], constants.gather_idx).cast::<f32>();

  let y16a = calc_y(constants, &vec_r1, &vec_g1, &vec_b1);
  let cb16 = calc_cb(constants, &vec_r1, &vec_g1, &vec_b1);
  let y16b = calc_y(constants, &vec_r2, &vec_g2, &vec_b2);
  let cr16 = calc_cr(constants, &vec_r1, &vec_g1, &vec_b1);

  let a1 = alpha_8_to_10bit(constants, &vec_a1);
  let a2 = alpha_8_to_10bit(constants, &vec_a2);

  let block1 = combine_components(constants, &a1, &cb16, &y16a);
  let block2 = combine_components(constants, &a2, &cr16, &y16b);

  for i in 0..4 {
    let offset = i * 8;
    let offset4 = offset + 4;
    let offset8 = offset + 8;

    target[offset..offset4].copy_from_slice(&block1[i].to_be_bytes());
    target[offset4..offset8].copy_from_slice(&block2[i].to_be_bytes())
  }
}

#[inline(always)]
fn calc_y(
  constants: &YuvConstantsSimd,
  r: &Simd<f32, 4>,
  g: &Simd<f32, 4>,
  b: &Simd<f32, 4>,
) -> Simd<f32, 4> {
  let luma = constants.kr * r + constants.kg * g + constants.kb * b;

  constants.luma_offset + (constants.luma_scale * luma)
}

#[inline(always)]
fn calc_cb(
  constants: &YuvConstantsSimd,
  r: &Simd<f32, 4>,
  g: &Simd<f32, 4>,
  b: &Simd<f32, 4>,
) -> Simd<f32, 4> {
  let val = constants.kr_o_kb_i * r + constants.kg_o_kb_i * g + b;

  constants.cb_cr_offset + (constants.half_cb_cr_scale * val)
}

#[inline(always)]
fn calc_cr(
  constants: &YuvConstantsSimd,
  r: &Simd<f32, 4>,
  g: &Simd<f32, 4>,
  b: &Simd<f32, 4>,
) -> Simd<f32, 4> {
  let val = r + constants.kg_o_kr_i * g + constants.kb_o_kr_i * b;

  constants.cb_cr_offset + (constants.half_cb_cr_scale * val)
}

#[inline(always)]
fn alpha_8_to_10bit(constants: &YuvConstantsSimd, val: &Simd<f32, 4>) -> Simd<f32, 4> {
  constants.alpha_offset + (val * constants.alpha_scale)
}

#[inline(always)]
fn combine_components(
  constants: &YuvConstantsSimd,
  a: &Simd<f32, 4>,
  uv: &Simd<f32, 4>,
  y: &Simd<f32, 4>,
) -> Simd<u32, 4> {
  // TODO - round these values?
  let a2 = a.cast::<u32>();
  let uv2 = uv.cast::<u32>();
  let y2 = y.cast::<u32>();

  (a2 << constants.shift_20) + (uv2 << constants.shift_10) + y2
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  fn rgb_to_yuv422_single(input: &[u8; 8]) -> [u8; 8] {
    let bt601_constants = YuvConstantsSimd::create(0.299, 0.114);

    let mut input_ext = [0; 32];
    input_ext[0..8].copy_from_slice(input);
    input_ext[8..16].copy_from_slice(input);
    input_ext[16..24].copy_from_slice(input);
    input_ext[24..32].copy_from_slice(input);

    let mut target = [0; 32];
    rgb_to_yuva422_simd(&bt601_constants, &input_ext, &mut target);

    let mut target_trimmed = [0; 8];
    target_trimmed.copy_from_slice(&target[0..8]);

    assert_eq!(&target[0..8], &target[8..16]);
    assert_eq!(&target[0..8], &target[16..24]);
    assert_eq!(&target[0..8], &target[24..32]);

    target_trimmed
  }

  #[test]
  fn test_black() {
    let input = [0, 0, 0, 0, 0, 0, 0, 0];
    let output = [4, 8, 0, 64, 4, 8, 0, 64];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_one() {
    let input = [28, 69, 148, 247, 117, 221, 18, 95];
    let output = [57, 10, 137, 32, 24, 102, 134, 122];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_two() {
    let input = [161, 62, 67, 203, 195, 251, 198, 239];
    let output = [47, 151, 57, 123, 55, 90, 175, 76];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_three() {
    let input = [189, 218, 98, 133, 76, 128, 210, 222];
    let output = [32, 132, 254, 221, 51, 167, 189, 224];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_four() {
    let input = [105, 85, 41, 102, 106, 19, 8, 133];
    let output = [25, 230, 157, 102, 32, 136, 188, 213];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_five() {
    let input = [96, 119, 88, 3, 90, 181, 110, 189];
    let output = [4, 167, 93, 179, 44, 151, 130, 50];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_six() {
    let input = [74, 254, 208, 22, 141, 123, 40, 132];
    let output = [8, 184, 102, 219, 32, 83, 73, 214];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }

  #[test]
  fn test_seven() {
    let input = [73, 49, 151, 173, 119, 123, 207, 152];
    let output = [41, 42, 145, 40, 36, 168, 50, 1];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }
  #[test]
  fn test_eight() {
    let input = [171, 98, 86, 67, 159, 235, 17, 105];
    let output = [18, 102, 253, 213, 26, 138, 14, 193];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }
  #[test]
  fn test_nine() {
    let input = [130, 226, 137, 84, 215, 97, 152, 118];
    let output = [22, 6, 114, 192, 29, 85, 198, 26];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }
  #[test]
  fn test_ten() {
    let input = [245, 20, 254, 158, 241, 114, 121, 152];
    let output = [37, 236, 81, 197, 36, 173, 30, 74];
    assert_eq!(rgb_to_yuv422_single(&input), output);
  }
}
