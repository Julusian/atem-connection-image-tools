use crate::yuv_constants::YuvConstantsSimd;
use std::simd::{num::SimdFloat, num::SimdUint, u32x4, Simd, StdFloat};

#[inline(always)]
pub fn yuva422_to_rgb_simd(
  constants: &YuvConstantsSimd,
  sample1: &[u8],
  sample2: &[u8],
  sample3: &[u8],
  sample4: &[u8],
  target: &mut [u8],
) {
  let ycba1_1 = u32::from_be_bytes((&sample1[0..4]).try_into().unwrap());
  let ycra1_2 = u32::from_be_bytes((&sample1[4..8]).try_into().unwrap());
  let ycba2_1 = u32::from_be_bytes((&sample2[0..4]).try_into().unwrap());
  let ycra2_2 = u32::from_be_bytes((&sample2[4..8]).try_into().unwrap());
  let ycba3_1 = u32::from_be_bytes((&sample3[0..4]).try_into().unwrap());
  let ycra3_2 = u32::from_be_bytes((&sample3[4..8]).try_into().unwrap());
  let ycba4_1 = u32::from_be_bytes((&sample4[0..4]).try_into().unwrap());
  let ycra4_2 = u32::from_be_bytes((&sample4[4..8]).try_into().unwrap());

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

  let px1 = pack_rgba(constants, &r1, &g1, &b1, &a1);
  let px2 = pack_rgba(constants, &r2, &g2, &b2, &a2);

  for i in 0..4 {
    let offset = i * 8;
    target[offset..offset + 4].copy_from_slice(&px1[i].to_le_bytes());
    target[offset + 4..offset + 8].copy_from_slice(&px2[i].to_le_bytes());
  }
}

/// Pack the channels into u32 words, which are RGBA when written as little endian
#[inline(always)]
fn pack_rgba(
  constants: &YuvConstantsSimd,
  r: &Simd<f32, 4>,
  g: &Simd<f32, 4>,
  b: &Simd<f32, 4>,
  a: &Simd<f32, 4>,
) -> Simd<u32, 4> {
  // r/g/b are already clamped to 0-255, alpha is clamped here
  let a = a.simd_clamp(constants.splat0f, constants.splat255f);

  // Safety: all values are finite and within 0-255, so fit in a u32
  let (r, g, b, a) = unsafe {
    (
      r.round().to_int_unchecked::<u32>(),
      g.round().to_int_unchecked::<u32>(),
      b.round().to_int_unchecked::<u32>(),
      a.round().to_int_unchecked::<u32>(),
    )
  };

  r | (g << 8) | (b << 16) | (a << 24)
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

    let mut target = [0; 32];
    yuva422_to_rgb_simd(&bt601_constants, input, input, input, input, &mut target);

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

  fn alpha_pair(alpha10: u32) -> [u8; 8] {
    let word = ((alpha10 << 20) | (512 << 10) | 64).to_be_bytes();
    let mut input = [0; 8];
    input[0..4].copy_from_slice(&word);
    input[4..8].copy_from_slice(&word);
    input
  }

  #[test]
  fn test_alpha_clamped() {
    // Stills captured on the switcher can use values outside of 64-940
    for (alpha10, alpha8) in [(0, 0), (63, 0), (944, 255), (1023, 255)] {
      let output = yuva422_to_rgb_single(&alpha_pair(alpha10));
      assert_eq!(
        (output[3], output[7]),
        (alpha8, alpha8),
        "alpha {}",
        alpha10
      );
    }
  }

  #[test]
  fn test_alpha_matches_switcher() {
    // Pairs observed from a still uploaded to a Constellation HD
    let observed: [(u8, u32); 7] = [
      (0, 64),
      (1, 67),
      (2, 71),
      (127, 500),
      (128, 504),
      (254, 937),
      (255, 940),
    ];
    for (alpha8, alpha10) in observed {
      let output = yuva422_to_rgb_single(&alpha_pair(alpha10));
      assert_eq!(
        (output[3], output[7]),
        (alpha8, alpha8),
        "alpha {}",
        alpha10
      );
    }
  }

  #[test]
  fn test_alpha_round_trip() {
    let bt601_constants = YuvConstantsSimd::create(0.299, 0.114);
    for alpha8 in 0..=255u8 {
      let mut rgba = [0u8; 32];
      for i in 0..8 {
        rgba[i * 4 + 3] = alpha8;
      }
      let mut yuva = [0u8; 32];
      crate::rgba_to_yuva422_simd::rgb_to_yuva422_simd(&bt601_constants, &rgba, &mut yuva);
      let mut result = [0u8; 32];
      yuva422_to_rgb_simd(
        &bt601_constants,
        &yuva[0..8],
        &yuva[8..16],
        &yuva[16..24],
        &yuva[24..32],
        &mut result,
      );
      for i in 0..8 {
        assert_eq!(result[i * 4 + 3], alpha8, "alpha {}", alpha8);
      }
    }
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
