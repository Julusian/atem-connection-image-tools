use crate::util::{copy_word, words_equal, Word64Bit};

const RLE_HEADER: Word64Bit = ([0xfe, 0xfe, 0xfe, 0xfe], [0xfe, 0xfe, 0xfe, 0xfe]);

pub struct RLEEncoder<'a> {
  output: &'a mut [u8],
  output_offset: usize,
  last_word: Word64Bit,
  match_count: u32,
  perform_rle: bool,
}
impl<'a> RLEEncoder<'a> {
  pub fn create(output: &mut [u8], perform_rle: bool) -> RLEEncoder {
    RLEEncoder {
      output,
      output_offset: 0,
      last_word: ([0, 0, 0, 0], [0, 0, 0, 0]),
      match_count: 0,
      perform_rle,
    }
  }

  fn write_last_block(&mut self) {
    match self.match_count {
      0 => {
        // Ignore
      }
      1 => {
        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;
      }
      2 => {
        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;

        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;
      }
      3 => {
        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;

        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;

        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;
      }
      _ => {
        copy_word(&mut self.output[self.output_offset..], &RLE_HEADER);
        self.output_offset += 8;

        let count = ([0, 0, 0, 0], self.match_count.to_be_bytes());
        copy_word(&mut self.output[self.output_offset..], &count);
        self.output_offset += 8;

        copy_word(&mut self.output[self.output_offset..], &self.last_word);
        self.output_offset += 8;
      }
    }
  }

  pub fn add_word(&mut self, word: Word64Bit) {
    if self.perform_rle {
      if words_equal(&word, &self.last_word) {
        self.match_count += 1;
      } else {
        self.write_last_block();
        self.last_word = word;
        self.match_count = 1;
      }
    } else {
      self.write_last_block();
      self.last_word = word;
      self.match_count = 1;
    }
  }
  pub fn finish(mut self) -> u32 {
    self.write_last_block();

    self.output_offset as u32
  }
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  //   const BLACK: Word64Bit = ([0, 0, 0, 0], [0, 0, 0, 0]);

  #[test]
  fn no_repititions() {
    let mut output = [0; 32];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([17, 18, 19, 20], [21, 22, 23, 24]));
    encoder.add_word(([25, 26, 27, 28], [29, 30, 31, 32]));

    assert_eq!(encoder.finish(), 32);
    assert_eq!(
      output,
      [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32
      ]
    )
  }

  #[test]
  fn single_repitition() {
    let mut output = [0; 32];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([17, 18, 19, 20], [21, 22, 23, 24]));

    assert_eq!(encoder.finish(), 32);
    assert_eq!(
      output,
      [
        1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        20, 21, 22, 23, 24,
      ]
    )
  }

  #[test]
  fn double_repitition() {
    let mut output = [0; 32];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));

    assert_eq!(encoder.finish(), 32);
    assert_eq!(
      output,
      [
        1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
        14, 15, 16,
      ]
    )
  }

  #[test]
  fn repitition_at_beginning() {
    let mut output = [0; 32];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));

    assert_eq!(encoder.finish(), 32);
    assert_eq!(
      output,
      [
        0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0, 0, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4, 5, 6,
        7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
      ]
    )
  }

  #[test]
  fn repitition_at_middle() {
    let mut output = [0; 40];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));

    assert_eq!(encoder.finish(), 40);
    assert_eq!(
      output,
      [
        9, 10, 11, 12, 13, 14, 15, 16, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0, 0, 0, 0,
        0, 0, 0, 4, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
      ]
    )
  }

  #[test]
  fn repitition_at_end() {
    let mut output = [0; 32];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));

    assert_eq!(encoder.finish(), 32);
    assert_eq!(
      output,
      [
        9, 10, 11, 12, 13, 14, 15, 16, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0, 0, 0, 0,
        0, 0, 0, 4, 1, 2, 3, 4, 5, 6, 7, 8,
      ]
    )
  }

  #[test]
  fn multiple_repititions() {
    let mut output = [0; 48];
    let mut encoder = RLEEncoder::create(&mut output, true);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));

    assert_eq!(encoder.finish(), 48);
    assert_eq!(
      output,
      [
        0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0, 0, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4, 5, 6,
        7, 8, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0, 0, 0, 0, 0, 0, 0, 5, 9, 10, 11,
        12, 13, 14, 15, 16
      ]
    )
  }

  #[test]
  fn rle_disabled() {
    let mut output = [0; 40];
    let mut encoder = RLEEncoder::create(&mut output, false);

    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([1, 2, 3, 4], [5, 6, 7, 8]));
    encoder.add_word(([9, 10, 11, 12], [13, 14, 15, 16]));

    assert_eq!(encoder.finish(), 40);
    assert_eq!(
      output,
      [
        1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6,
        7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
      ]
    )
  }
}
