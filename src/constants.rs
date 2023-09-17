pub struct YuvConstants {
  pub kr: f32,
  pub kb: f32,
  pub kg: f32,

  // pub kr_i: f32,
  // pub kb_i: f32,
  pub kr_o_kb_i: f32,
  pub kg_o_kb_i: f32,
  pub kb_o_kr_i: f32,
  pub kg_o_kr_i: f32,
}
impl YuvConstants {
  pub fn create(kr: f32, kb: f32) -> YuvConstants {
    let kg = 1.0 - kr - kb;
    let kr_i = 1.0 - kr;
    let kb_i = 1.0 - kb;

    YuvConstants {
      kr,
      kb,
      kg,

      // kr_i,
      // kb_i,
      kr_o_kb_i: kr / kb_i,
      kg_o_kb_i: kg / kb_i,
      kb_o_kr_i: kb / kr_i,
      kg_o_kr_i: kg / kr_i,
    }
  }
}
