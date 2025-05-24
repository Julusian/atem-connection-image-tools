use napi::JsBufferValue;

pub(crate) struct RLEDecoder<'a> {
  input: &'a JsBufferValue,
  read_offset: usize,
  rle_remaining: usize,
}
impl<'a> RLEDecoder<'a> {
  pub fn new(input: &'a JsBufferValue) -> Self {
    RLEDecoder {
      input,
      read_offset: 0,
      rle_remaining: 0,
    }
  }

  pub fn read_sample(&mut self) -> Option<&'a [u8]> {
    // Make sure we aren't beyond the end of the input
    if self.read_offset + 8 > self.input.len() {
      return None; // No more data to read
    }

    // In the middle of an RLE sequence, so repeat the value
    if self.rle_remaining > 0 {
      self.rle_remaining -= 1;

      // If Not the last sample, read the sample without moving the offset
      if self.rle_remaining > 0 {
        return Some(&self.input[self.read_offset..self.read_offset + 8]);
      }

      // The last sample, handle with the default case
    } else
    // Check for the start of a RLE sequence
    if self.input.len() >= self.read_offset + 24 // this will take 3 samples
      && self.input[self.read_offset] == 0xfe
      && self.input[self.read_offset + 1] == 0xfe
      && self.input[self.read_offset + 2] == 0xfe
      && self.input[self.read_offset + 3] == 0xfe
      && self.input[self.read_offset + 4] == 0xfe
      && self.input[self.read_offset + 5] == 0xfe
      && self.input[self.read_offset + 6] == 0xfe
      && self.input[self.read_offset + 7] == 0xfe
    {
      self.rle_remaining = u64::from_be_bytes(
        self.input[self.read_offset + 8..self.read_offset + 16]
          .try_into()
          .unwrap(),
      ) as usize
        - 1; // -1 because we will read the first sample now

      self.read_offset += 16; // Move past the RLE header

      return Some(&self.input[self.read_offset..self.read_offset + 8]);
    }

    // Read a normal sample
    let value = &self.input[self.read_offset..self.read_offset + 8];

    self.read_offset += 8; // Move past the sample

    return Some(value);
  }
}
