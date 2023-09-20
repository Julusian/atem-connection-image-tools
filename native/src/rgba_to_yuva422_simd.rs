use std::simd::{f32x4, u32x4, Simd};

use crate::util::Word64Bit;

pub struct YuvConstantsSimd {
  pub kr: Simd<f32, 4>,
  pub kb: Simd<f32, 4>,
  pub kg: Simd<f32, 4>,

  pub kr_o_kb_i: Simd<f32, 4>,
  pub kg_o_kb_i: Simd<f32, 4>,
  pub kb_o_kr_i: Simd<f32, 4>,
  pub kg_o_kr_i: Simd<f32, 4>,

  pub luma_scale: Simd<f32, 4>,
  pub luma_offset: Simd<f32, 4>,

  pub cb_cr_offset: Simd<f32, 4>,
  pub half_cb_cr_scale: Simd<f32, 4>,

  pub alpha_scale: Simd<f32, 4>,
  pub alpha_offset: Simd<f32, 4>,

  pub shift_20: Simd<u32, 4>,
  pub shift_10: Simd<u32, 4>,

  pub splat2: Simd<u32, 4>,
  pub splat4: Simd<u32, 4>,
  pub splat6: Simd<u32, 4>,
  pub splat8: Simd<u32, 4>,
  pub splat15: Simd<u32, 4>,
  pub splat255: Simd<u32, 4>,
}
impl YuvConstantsSimd {
  pub fn create(kr: f32, kb: f32) -> YuvConstantsSimd {
    let kg = 1.0 - kr - kb;
    let kr_i = 1.0 - kr;
    let kb_i = 1.0 - kb;

    YuvConstantsSimd {
      kr: f32x4::splat(kr),
      kb: f32x4::splat(kb),
      kg: f32x4::splat(kg),

      kr_o_kb_i: f32x4::splat(-kr / kb_i),
      kg_o_kb_i: f32x4::splat(-kg / kb_i),
      kb_o_kr_i: f32x4::splat(-kb / kr_i),
      kg_o_kr_i: f32x4::splat(-kg / kr_i),

      luma_scale: f32x4::splat(219.0 / 64.0),
      luma_offset: f32x4::splat(64.0),

      cb_cr_offset: f32x4::splat(512.0),
      half_cb_cr_scale: f32x4::splat(224.0 / 64.0 / 2.0),

      alpha_scale: f32x4::splat(219.0 / 255.0 * 4.0),
      alpha_offset: f32x4::splat(64.0),

      shift_20: u32x4::splat(20),
      shift_10: u32x4::splat(10),

      splat2: u32x4::splat(2),
      splat4: u32x4::splat(4),
      splat6: u32x4::splat(6),
      splat8: u32x4::splat(8),
      splat15: u32x4::splat(0x0f),
      splat255: u32x4::splat(0xff),
    }
  }
}

fn f32x4_from_u8(v1: u8, v2: u8, v3: u8, v4: u8) -> Simd<f32, 4> {
  f32x4::from_array([v1 as f32, v2 as f32, v3 as f32, v4 as f32])
}

#[inline(always)]
pub fn rgb_to_yuv422_simd(
  constants: &YuvConstantsSimd,
  input: &[u8],
  // target: &mut [u8],
  // perform_rle: bool,
  // previous_target: Option<&[u8]>,
) -> (Word64Bit, Word64Bit, Word64Bit, Word64Bit) {
  let rgba1_1 = &input[0..4];
  let rgba1_2 = &input[4..8];
  let rgba2_1 = &input[8..12];
  let rgba2_2 = &input[12..16];
  let rgba3_1 = &input[16..20];
  let rgba3_2 = &input[20..24];
  let rgba4_1 = &input[24..28];
  let rgba4_2 = &input[28..32];

  let vec_r1 = f32x4_from_u8(rgba1_1[0], rgba2_1[0], rgba3_1[0], rgba4_1[0]);
  let vec_g1 = f32x4_from_u8(rgba1_1[1], rgba2_1[1], rgba3_1[1], rgba4_1[1]);
  let vec_b1 = f32x4_from_u8(rgba1_1[2], rgba2_1[2], rgba3_1[2], rgba4_1[2]);
  let vec_a1 = f32x4_from_u8(rgba1_1[3], rgba2_1[3], rgba3_1[3], rgba4_1[3]);

  let vec_r2 = f32x4_from_u8(rgba1_2[0], rgba2_2[0], rgba3_2[0], rgba4_2[0]);
  let vec_g2 = f32x4_from_u8(rgba1_2[1], rgba2_2[1], rgba3_2[1], rgba4_2[1]);
  let vec_b2 = f32x4_from_u8(rgba1_2[2], rgba2_2[2], rgba3_2[2], rgba4_2[2]);
  let vec_a2 = f32x4_from_u8(rgba1_2[3], rgba2_2[3], rgba3_2[3], rgba4_2[3]);

  let y16a = calc_y(constants, &vec_r1, &vec_g1, &vec_b1);
  let cb16 = calc_cb(constants, &vec_r1, &vec_g1, &vec_b1);
  let y16b = calc_y(constants, &vec_r2, &vec_g2, &vec_b2);
  let cr16 = calc_cr(constants, &vec_r1, &vec_g1, &vec_b1);

  let a1 = alpha_8_to_10bit(constants, &vec_a1);
  let a2 = alpha_8_to_10bit(constants, &vec_a2);

  let block1 = combine_components(constants, &a1, &cb16, &y16a);
  let block2 = combine_components(constants, &a2, &cr16, &y16b);

  let word1 = (block1[0].to_be_bytes(), block2[0].to_be_bytes());
  let word2 = (block1[1].to_be_bytes(), block2[1].to_be_bytes());
  let word3 = (block1[2].to_be_bytes(), block2[2].to_be_bytes());
  let word4 = (block1[3].to_be_bytes(), block2[3].to_be_bytes());

  (word1, word2, word3, word4)
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
fn to_simd_u32(input: &Simd<f32, 4>) -> Simd<u32, 4> {
  u32x4::from_array([
    input[0] as u32,
    input[1] as u32,
    input[2] as u32,
    input[3] as u32,
  ])
}

#[inline(always)]
fn combine_components(
  constants: &YuvConstantsSimd,
  a: &Simd<f32, 4>,
  uv: &Simd<f32, 4>,
  y: &Simd<f32, 4>,
) -> Simd<u32, 4> {
  // TODO - round these values?
  let a2 = to_simd_u32(a);
  let uv2 = to_simd_u32(uv);
  let y2 = to_simd_u32(y);

  (a2 << constants.shift_20) + (uv2 << constants.shift_10) + y2
}

#[cfg(test)]
mod tests {
  use crate::util::copy_all;

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
    let (word1, word2, word3, word4) = rgb_to_yuv422_simd(&bt601_constants, &input_ext);
    copy_all(&mut target, &word1, &word2, &word3, &word4);

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
