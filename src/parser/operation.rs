use crate::util;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationKind {
  Unknown,
  Create,
  Remove,
  Add,
  Show,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation {
  kind: OperationKind,
  department: Option<String>,
  fail_silently: Option<bool>,
  names: Option<Vec<String>>,
  overwrite: Option<bool>,
}

use self::OperationKind::*;

impl Operation {
  pub fn unknown() -> Self {
    Operation {
      kind: Unknown,
      department: None,
      fail_silently: None,
      names: None,
      overwrite: None,
    }
  }

  pub fn create(department: String, fail_silently: bool, overwrite: bool) -> Self {
    Operation {
      kind: Create,
      department: Some(department),
      fail_silently: Some(fail_silently),
      overwrite: Some(overwrite),
      ..Self::unknown()
    }
  }

  pub fn remove(department: String, fail_silently: bool, names: Vec<String>) -> Self {
    Operation {
      kind: Remove,
      department: Some(department),
      fail_silently: Some(fail_silently),
      names: Some(names),
      ..Self::unknown()
    }
  }

  pub fn add(department: String, fail_silently: bool, names: Vec<String>, overwrite: bool) -> Self {
    Operation {
      kind: Add,
      department: Some(department),
      fail_silently: Some(fail_silently),
      names: Some(names),
      overwrite: Some(overwrite),
      ..Self::unknown()
    }
  }

  pub fn show(department: String, fail_silently: bool) -> Self {
    Operation {
      kind: Show,
      department: Some(department),
      fail_silently: Some(fail_silently),
      ..Self::unknown()
    }
  }

  pub fn kind(&self) -> OperationKind {
    self.kind
  }

  pub fn get_department(&self) -> Option<&str> {
    self.department.as_deref()
  }

  pub fn department(&self) -> &str {
    self.get_department().unwrap()
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

  pub fn set_department(self, department: String) -> Option<Self> {
    if let Some(_) = self.department {
      Some(Self {
        department: Some(department),
        ..self
      })
    } else {
      None
    }
  }

  pub fn set_fail_silently(self, fail_silently: bool) -> Option<Self> {
    if let Some(_) = self.fail_silently {
      Some(Self {
        fail_silently: Some(fail_silently),
        ..self
      })
    } else {
      None
    }
  }

  pub fn set_names(self, names: Vec<String>) -> Option<Self> {
    if let Some(_) = self.names {
      Some(Self {
        names: Some(names),
        ..self
      })
    } else {
      None
    }
  }

  pub fn set_overwrite(self, overwrite: bool) -> Option<Self> {
    if let Some(_) = self.overwrite {
      Some(Self {
        overwrite: Some(overwrite),
        ..self
      })
    } else {
      None
    }
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

pub fn fmt_names<'a>(elems: &'a [String], linker: &str) -> Cow<'a, str> {
  let names = util::fmt_list(elems, ", ", "and");
  if names.is_empty() {
    "".into()
  } else {
    format!(" {} {}", names, linker).into()
  }
}

impl fmt::Display for Operation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let statement: Cow<str> = match self.kind {
      Unknown => return f.write_str("Unknown"),
      Create => "Create".into(),
      Show => "Show".into(),
      Add => format!("Add{}", fmt_names(self.names(), "to")).into(),
      Remove => format!("Remove{}", fmt_names(self.names(), "from")).into(),
    };
    write!(
      f,
      "{} {}{}",
      statement,
      self.department(),
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
          "Science".into(),
          false,
          vec!["Mama".into(), "Tata".into(), "Bunica Miha".into()],
          false,
        ),
        "Add Mama, Tata, and Bunica Miha to Science",
      ),
      (
        Operation::remove("Engineering".into(), true, vec!["Sally".into()]),
        "Remove Sally from Engineering (fail silently)",
      ),
      (
        Operation::create("Sales".into(), false, true),
        "Create Sales (overwrite if existing)",
      ),
      (Operation::show("HR".into(), false), "Show HR"),
    ];

    ops
      .iter()
      .for_each(|(op, expect)| assert_eq!(format!("{}", op), *expect));
  }
}