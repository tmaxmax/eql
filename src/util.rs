use unic_ucd_category::GeneralCategory;
use unicode_segmentation::UnicodeSegmentation;

pub fn repeat_chars(ch: &str, n: usize) -> String {
  std::iter::repeat(ch).take(n).collect()
}

pub fn is_alphabetic(s: &str) -> bool {
  s.chars().map(GeneralCategory::of).all(|c| c.is_letter())
}

//pub fn is_numeric(s: &str) -> bool {
//  s.chars().map(GeneralCategory::of).all(|c| c.is_number())
//}

//pub fn is_separator(s: &str) -> bool {
//  s.chars().map(GeneralCategory::of).all(|c| c.is_separator())
//}

pub fn is_whitespace(s: &str) -> bool {
  s.chars().all(char::is_whitespace)
}

//pub fn is_punctuation(s: &str) -> bool {
//  s.chars()
//    .map(GeneralCategory::of)
//    .all(|c| c.is_punctuation())
//}

pub fn string_length(word: &str) -> usize {
  word.graphemes(true).count()
}
