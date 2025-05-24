use std::simd::{f32x4, u32x4, Simd};

pub struct YuvConstantsSimd {
  pub kr: Simd<f32, 4>,
  pub kb: Simd<f32, 4>,
  pub kg: Simd<f32, 4>,

  pub kr_i: Simd<f32, 4>,
  pub kb_i: Simd<f32, 4>,

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

  pub cr_to_g: Simd<f32, 4>,
  pub cb_to_g: Simd<f32, 4>,

  pub shift_20: Simd<u32, 4>,
  pub shift_10: Simd<u32, 4>,

  pub splat2: Simd<u32, 4>,
  pub splat4: Simd<u32, 4>,
  pub splat6: Simd<u32, 4>,
  pub splat8: Simd<u32, 4>,
  pub splat15: Simd<u32, 4>,
  pub splat255: Simd<u32, 4>,
  pub splat1023: Simd<u32, 4>,

  pub splat0f: Simd<f32, 4>,
  pub splat255f: Simd<f32, 4>,

  pub scatter_idx: Simd<usize, 4>,
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

      kr_i: f32x4::splat(kr_i),
      kb_i: f32x4::splat(kb_i),

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

      cr_to_g: f32x4::splat(kr * kr_i / kg),
      cb_to_g: f32x4::splat(kb * kb_i / kg),

      shift_20: u32x4::splat(20),
      shift_10: u32x4::splat(10),

      splat2: u32x4::splat(2),
      splat4: u32x4::splat(4),
      splat6: u32x4::splat(6),
      splat8: u32x4::splat(8),
      splat15: u32x4::splat(0x0f),
      splat255: u32x4::splat(0xff),
      splat1023: u32x4::splat(0x3ff),

      splat0f: f32x4::splat(0.0),
      splat255f: f32x4::splat(255.0),

      scatter_idx: Simd::from_array([0, 8, 16, 24]),
    }
  }
}
