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
  pub fn create(output: &mut [u8], first_word: Word64Bit, perform_rle: bool) -> RLEEncoder {
    RLEEncoder {
      output,
      output_offset: 0,
      last_word: first_word,
      match_count: 1,
      perform_rle,
    }
  }

  fn write_last_block(&mut self) {
    if self.match_count <= 1 {
      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;
    } else if self.match_count == 2 {
      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;

      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;
    } else if self.match_count == 3 {
      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;

      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;

      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;
    } else {
      copy_word(&mut self.output[self.output_offset..], &RLE_HEADER);
      self.output_offset += 8;

      let count = ([0, 0, 0, 0], self.match_count.to_be_bytes());
      copy_word(&mut self.output[self.output_offset..], &count);
      self.output_offset += 8;

      copy_word(&mut self.output[self.output_offset..], &self.last_word);
      self.output_offset += 8;
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
    }
  }
  pub fn finish(mut self) -> u32 {
    self.write_last_block();

    self.output_offset as u32
  }
}
