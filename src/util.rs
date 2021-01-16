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

pub fn fmt_token_pointer(token_value: &str, col: usize) -> (String, String) {
  (
    repeat_chars(" ", col - 1),
    repeat_chars("^", string_length(token_value)),
  )
}

pub fn fmt_list<'a, T: std::fmt::Display>(elems: &'a [T], sep: &str, linker: &str) -> String {
  if let [rest @ .., last] = elems {
    if rest.is_empty() {
      format!("{}", last)
    } else {
      format!(
        "{}{} {} {}",
        rest.iter().map(T::to_string).collect::<Vec<_>>().join(sep),
        if rest.len() == 1 { "" } else { "," },
        linker,
        last
      )
    }
  } else {
    "".into()
  }
}

pub fn to_string_vec(v: Vec<&str>) -> Vec<String> {
  v.into_iter().map(String::from).collect()
}
