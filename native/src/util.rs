use std::simd::{f32x4, u32x4, Simd};

#[inline(always)]
pub fn f32x4_from_u8(v1: u8, v2: u8, v3: u8, v4: u8) -> Simd<f32, 4> {
  f32x4::from_array([v1 as f32, v2 as f32, v3 as f32, v4 as f32])
}

#[inline(always)]
pub fn to_simd_u32(input: &Simd<f32, 4>) -> Simd<u32, 4> {
  u32x4::from_array([
    input[0] as u32,
    input[1] as u32,
    input[2] as u32,
    input[3] as u32,
  ])
}

#[inline(always)]
pub fn to_simd_f32(input: &Simd<u32, 4>) -> Simd<f32, 4> {
  f32x4::from_array([
    input[0] as f32,
    input[1] as f32,
    input[2] as f32,
    input[3] as f32,
  ])
}
