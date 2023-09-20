pub type Word64Bit = ([u8; 4], [u8; 4]);

#[inline(always)]
pub fn words_equal(word1: &Word64Bit, word2: &Word64Bit) -> bool {
  word1.0.eq(&word2.0) && word1.1.eq(&word2.1)
}

#[inline(always)]
pub fn copy_all(
  target: &mut [u8],
  word1: &Word64Bit,
  word2: &Word64Bit,
  word3: &Word64Bit,
  word4: &Word64Bit,
) {
  copy_word(target, word1);
  copy_word(&mut target[8..], word2);
  copy_word(&mut target[16..], word3);
  copy_word(&mut target[24..], word4);
}

#[inline(always)]
pub fn copy_word(target: &mut [u8], word: &Word64Bit) {
  target[0..4].copy_from_slice(&word.0);
  target[4..8].copy_from_slice(&word.1);
}
