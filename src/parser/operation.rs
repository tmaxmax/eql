use crate::util;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationKind {
  Unknown,
  Create,
  Remove,
  Add,
  Show,
}

impl fmt::Display for OperationKind {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.write_str(match *self {
      Unknown => "Unknown",
      Create => "Create",
      Remove => "Remove",
      Add => "Add",
      Show => "Show",
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation {
  kind: OperationKind,
  departments: Option<Vec<String>>,
  fail_silently: Option<bool>,
  names: Option<Vec<String>>,
  overwrite: Option<bool>,
}

use self::OperationKind::*;

impl Operation {
  pub fn unknown() -> Self {
    Self {
      kind: Unknown,
      departments: None,
      fail_silently: None,
      names: None,
      overwrite: None,
    }
  }

  pub fn create(departments: Vec<String>, fail_silently: bool, overwrite: bool) -> Self {
    Self {
      kind: Create,
      departments: Some(departments),
      fail_silently: Some(fail_silently),
      overwrite: Some(overwrite),
      ..Self::unknown()
    }
  }

  pub fn remove(departments: Vec<String>, fail_silently: bool, names: Vec<String>) -> Self {
    Self {
      kind: Remove,
      departments: Some(departments),
      fail_silently: Some(fail_silently),
      names: Some(names),
      ..Self::unknown()
    }
  }

  pub fn add(
    departments: Vec<String>,
    fail_silently: bool,
    names: Vec<String>,
    overwrite: bool,
  ) -> Self {
    Self {
      kind: Add,
      departments: Some(departments),
      fail_silently: Some(fail_silently),
      names: Some(names),
      overwrite: Some(overwrite),
    }
  }

  pub fn show(departments: Vec<String>, fail_silently: bool) -> Self {
    Self {
      kind: Show,
      departments: Some(departments),
      fail_silently: Some(fail_silently),
      ..Self::unknown()
    }
  }

  pub fn kind(&self) -> OperationKind {
    self.kind
  }

  pub fn get_departments(&self) -> Option<&[String]> {
    self.departments.as_deref()
  }

  pub fn departments(&self) -> &[String] {
    self.get_departments().unwrap()
  }

  pub fn get_fail_silently(&self) -> Option<bool> {
    self.fail_silently
  }

  pub fn fail_silently(&self) -> bool {
    self.get_fail_silently().unwrap()
  }

  pub fn get_names(&self) -> Option<&[String]> {
    self.names.as_deref()
  }

  pub fn names(&self) -> &[String] {
    self.get_names().unwrap()
  }

  pub fn get_overwrite(&self) -> Option<bool> {
    self.overwrite
  }

  pub fn overwrite(&self) -> bool {
    self.get_overwrite().unwrap()
  }

  pub fn set_departments(self, departments: Vec<String>) -> Option<Self> {
    self.departments.and(Some(Self {
      departments: Some(departments),
      ..self
    }))
  }

  pub fn set_fail_silently(self, fail_silently: bool) -> Option<Self> {
    self.fail_silently.and(Some(Self {
      fail_silently: Some(fail_silently),
      ..self
    }))
  }

  pub fn set_names(self, names: Vec<String>) -> Option<Self> {
    self.names.and(Some(Self {
      names: Some(names),
      ..self
    }))
  }

  pub fn set_overwrite(self, overwrite: bool) -> Option<Self> {
    self.overwrite.and(Some(Self {
      overwrite: Some(overwrite),
      ..self
    }))
  }
}

fn fmt_modifier(op: &Operation) -> &'static str {
  if op.get_fail_silently().unwrap_or_default() {
    " (fail silently)"
  } else if op.get_overwrite().unwrap_or_default() {
    " (overwrite if existing)"
  } else {
    ""
  }
}

pub fn fmt_names(elems: &[String], linker: &str) -> String {
  let names = util::fmt_list(elems, ", ", "and");
  if names.is_empty() {
    names
  } else {
    format!(" {} {}", names, linker)
  }
}

impl fmt::Display for Operation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let statement = match self.kind() {
      Unknown => return self.kind().fmt(f),
      Create | Show => self.kind().to_string(),
      _ => format!(
        "{}{}",
        self.kind(),
        fmt_names(
          self.names(),
          match self.kind() {
            Add => "to",
            Remove => "from",
            _ => "",
          }
        )
      ),
    };
    write!(
      f,
      "{} {}{}",
      statement,
      util::fmt_list(self.departments(), ", ", "and"),
      fmt_modifier(self)
    )
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn format() {
    let ops = &[
      (Operation::unknown(), "Unknown"),
      (
        Operation::add(
          util::to_string_vec(vec!["Science", "Engineering"]),
          false,
          vec!["Mama".into(), "Tata".into(), "Bunica Miha".into()],
          false,
        ),
        "Add Mama, Tata, and Bunica Miha to Science and Engineering",
      ),
      (
        Operation::remove(
          util::to_string_vec(vec!["Engineering"]),
          true,
          vec!["Sally".into()],
        ),
        "Remove Sally from Engineering (fail silently)",
      ),
      (
        Operation::create(util::to_string_vec(vec!["Sales"]), false, true),
        "Create Sales (overwrite if existing)",
      ),
      (
        Operation::show(util::to_string_vec(vec!["HR"]), false),
        "Show HR",
      ),
    ];

    ops
      .iter()
      .for_each(|(op, expect)| assert_eq!(format!("{}", op), *expect));
  }
}
