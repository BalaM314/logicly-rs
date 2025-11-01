
pub fn bits_to_int<'a>(bits: impl DoubleEndedIterator<Item = &'a bool>) -> usize {
  bits.into_iter().fold(0, |acc, x| (acc << 1) + (*x as usize))
}
pub fn int_to_bits(int: usize, len: u8) -> Vec<bool> {
  let len = len as usize;
  (0..len).map(|i| int & (1 << (len - i - 1)) != 0).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test(){
    assert_eq!(bits_to_int(vec![true, false, false, true, true, true, true, true].iter()), 0b10011111);
    assert_eq!(bits_to_int(vec![false, false, false, true, true, true, true, false].iter()), 0b00011110);
    assert_eq!(bits_to_int(vec![true, true, true, true, false].iter()), 0b11110);
    assert_eq!(int_to_bits(0b10011111, 8), vec![true, false, false, true, true, true, true, true]);
    assert_eq!(int_to_bits(0b00011110, 8), vec![false, false, false, true, true, true, true, false]);
    assert_eq!(int_to_bits(0b00011110, 5), vec![true, true, true, true, false]);
  }
}
