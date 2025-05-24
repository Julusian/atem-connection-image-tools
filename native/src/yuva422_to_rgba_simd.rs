use crate::yuv_constants::YuvConstantsSimd;
use std::simd::{num::SimdFloat, num::SimdUint, u32x4, Simd, StdFloat};

#[inline(always)]
pub fn yuva422_to_rgb_simd(constants: &YuvConstantsSimd, input: &[u8], target: &mut [u8]) {
  let ycba1_1 = u32::from_be_bytes((&input[0..4]).try_into().unwrap());
  let ycra1_2 = u32::from_be_bytes((&input[4..8]).try_into().unwrap());
  let ycba2_1 = u32::from_be_bytes((&input[8..12]).try_into().unwrap());
  let ycra2_2 = u32::from_be_bytes((&input[12..16]).try_into().unwrap());
  let ycba3_1 = u32::from_be_bytes((&input[16..20]).try_into().unwrap());
  let ycra3_2 = u32::from_be_bytes((&input[20..24]).try_into().unwrap());
  let ycba4_1 = u32::from_be_bytes((&input[24..28]).try_into().unwrap());
  let ycra4_2 = u32::from_be_bytes((&input[28..32]).try_into().unwrap());

  let vec_ycba = u32x4::from_array([ycba1_1, ycba2_1, ycba3_1, ycba4_1]);
  let vec_ycra = u32x4::from_array([ycra1_2, ycra2_2, ycra3_2, ycra4_2]);

  let (vec_y1, vec_cb, a1) = split_components(constants, &vec_ycba);
  let (vec_y2, vec_cr, a2) = split_components(constants, &vec_ycra);

  let r1 = calc_r(constants, &vec_y1, &vec_cr);
  let g1 = calc_g(constants, &vec_y1, &vec_cb, &vec_cr);
  let b1 = calc_b(constants, &vec_y1, &vec_cb);

  let r2 = calc_r(constants, &vec_y2, &vec_cr);
  let g2 = calc_g(constants, &vec_y2, &vec_cb, &vec_cr);
  let b2 = calc_b(constants, &vec_y2, &vec_cb);

  let r1_u8 = r1.round().cast::<u8>();
  let g1_u8 = g1.round().cast::<u8>();
  let b1_u8 = b1.round().cast::<u8>();
  let a1_u8 = a1.round().cast::<u8>();

  let r2_u8 = r2.round().cast::<u8>();
  let g2_u8 = g2.round().cast::<u8>();
  let b2_u8 = b2.round().cast::<u8>();
  let a2_u8 = a2.round().cast::<u8>();

  r1_u8.scatter(target, constants.scatter_idx);
  g1_u8.scatter(&mut target[1..], constants.scatter_idx);
  b1_u8.scatter(&mut target[2..], constants.scatter_idx);
  a1_u8.scatter(&mut target[3..], constants.scatter_idx);

  r2_u8.scatter(&mut target[4..], constants.scatter_idx);
  g2_u8.scatter(&mut target[5..], constants.scatter_idx);
  b2_u8.scatter(&mut target[6..], constants.scatter_idx);
  a2_u8.scatter(&mut target[7..], constants.scatter_idx);
}

#[inline(always)]
fn calc_r(constants: &YuvConstantsSimd, y: &Simd<f32, 4>, cr: &Simd<f32, 4>) -> Simd<f32, 4> {
  let val = y + constants.kr_i * cr;
  val.simd_clamp(constants.splat0f, constants.splat255f)
}

#[inline(always)]
fn calc_g(
  constants: &YuvConstantsSimd,
  y: &Simd<f32, 4>,
  cb: &Simd<f32, 4>,
  cr: &Simd<f32, 4>,
) -> Simd<f32, 4> {
  let val = y - (constants.cr_to_g * cr) - (constants.cb_to_g * cb);
  val.simd_clamp(constants.splat0f, constants.splat255f)
}

#[inline(always)]
fn calc_b(constants: &YuvConstantsSimd, y: &Simd<f32, 4>, cb: &Simd<f32, 4>) -> Simd<f32, 4> {
  let val = y + constants.kb_i * cb;
  val.simd_clamp(constants.splat0f, constants.splat255f)
}

#[inline(always)]
fn split_components(
  constants: &YuvConstantsSimd,
  vec_combined: &Simd<u32, 4>,
) -> (Simd<f32, 4>, Simd<f32, 4>, Simd<f32, 4>) {
  let a = (vec_combined >> constants.shift_20) & constants.splat1023;
  let uv = (vec_combined >> constants.shift_10) & constants.splat1023;
  let y = vec_combined & constants.splat1023;

  let y_full = (y.cast::<f32>() - constants.luma_offset) / constants.luma_scale;
  let uv_full = (uv.cast::<f32>() - constants.cb_cr_offset) / constants.half_cb_cr_scale;
  let a_full = (a.cast::<f32>() - constants.alpha_offset) / constants.alpha_scale;

  (y_full, uv_full, a_full)
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  fn yuva422_to_rgb_single(input: &[u8; 8]) -> [u8; 8] {
    let bt601_constants = YuvConstantsSimd::create(0.299, 0.114);

    let mut input_ext = [0; 32];
    input_ext[0..8].copy_from_slice(input);
    input_ext[8..16].copy_from_slice(input);
    input_ext[16..24].copy_from_slice(input);
    input_ext[24..32].copy_from_slice(input);

    let mut target = [0; 32];
    yuva422_to_rgb_simd(&bt601_constants, &input_ext, &mut target);

    let mut target_trimmed = [0; 8];
    target_trimmed.copy_from_slice(&target[0..8]);

    assert_eq!(&target[0..8], &target[8..16]);
    assert_eq!(&target[0..8], &target[16..24]);
    assert_eq!(&target[0..8], &target[24..32]);

    target_trimmed
  }

  #[test]
  fn test_black() {
    let input = [4, 8, 0, 64, 4, 8, 0, 64];
    let output = [0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(yuva422_to_rgb_single(&input), output);
  }

  // TODO: are these tests any good?
  // They assume lossess cb/cr values, which is not the case because of the 422

  // #[test]
  // fn test_one() {
  //   let input = [57, 10, 137, 32, 24, 102, 134, 122];
  //   let output = [28, 69, 148, 247, 117, 221, 18, 95];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_two() {
  //   let input = [47, 151, 57, 123, 55, 90, 175, 76];
  //   let output = [161, 62, 67, 203, 195, 251, 198, 239];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_three() {
  //   let input = [32, 132, 254, 221, 51, 167, 189, 224];
  //   let output = [189, 218, 98, 133, 76, 128, 210, 222];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_four() {
  //   let input = [25, 230, 157, 102, 32, 136, 188, 213];
  //   let output = [105, 85, 41, 102, 106, 19, 8, 133];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_five() {
  //   let input = [4, 167, 93, 179, 44, 151, 130, 50];
  //   let output = [96, 119, 88, 3, 90, 181, 110, 189];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_six() {
  //   let input = [8, 184, 102, 219, 32, 83, 73, 214];
  //   let output = [74, 254, 208, 22, 141, 123, 40, 132];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }

  // #[test]
  // fn test_seven() {
  //   let input = [41, 42, 145, 40, 36, 168, 50, 1];
  //   let output = [73, 49, 151, 173, 119, 123, 207, 152];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }
  // #[test]
  // fn test_eight() {
  //   let input = [18, 102, 253, 213, 26, 138, 14, 193];
  //   let output = [171, 98, 86, 67, 159, 235, 17, 105];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }
  // #[test]
  // fn test_nine() {
  //   let input = [22, 6, 114, 192, 29, 85, 198, 26];
  //   let output = [130, 226, 137, 84, 215, 97, 152, 118];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }
  // #[test]
  // fn test_ten() {
  //   let input = [37, 236, 81, 197, 36, 173, 30, 74];
  //   let output = [245, 20, 254, 158, 241, 114, 121, 152];
  //   assert_eq!(yuva422_to_rgb_single(&input), output);
  // }
}
