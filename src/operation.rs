use crate::util;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifier {
  FailSilently,
  Overwrite,
  None,
}

impl fmt::Display for Modifier {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self {
      Modifier::FailSilently => "fail silently",
      Modifier::Overwrite => "overwrite if existing",
      Modifier::None => ""
    })
  }
}

fn fmt_list(list: &[String]) -> String {
  util::fmt_list(list, ", ", "and")
}

fn fmt_modifier(m: Modifier) -> String {
  let s = m.to_string();
  if s.is_empty() {
    s
  } else {
    format!(" ({})", s)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OperationAdd {
  pub names: Vec<String>,
  pub departments: Vec<String>,
  modifier: Modifier,
}

impl OperationAdd {
  #[inline]
  pub fn new(departments: Vec<String>, names: Vec<String>, modifier: Modifier) -> Self {
    Self {
      departments,
      names,
      modifier,
    }
  }

  pub const fn keyword() -> &'static str {
    "Add"
  }

  #[inline]
  pub fn modifier(&self) -> Modifier {
    self.modifier
  }

  #[inline]
  pub fn set_modifier(self, modifier: Modifier) -> Option<Self> {
    Some(Self {
      departments: self.departments,
      names: self.names,
      modifier,
    })
  }
}

impl fmt::Display for OperationAdd {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {} to {}{}", OperationAdd::keyword(), fmt_list(&self.names), fmt_list(&self.departments), fmt_modifier(self.modifier))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OperationCreate {
  pub departments: Vec<String>,
  modifier: Modifier,
}

impl OperationCreate {
  #[inline]
  pub fn new(departments: Vec<String>, modifier: Modifier) -> Self {
    Self {
      departments,
      modifier,
    }
  }

  pub const fn keyword() -> &'static str {
    "Create"
  }

  #[inline]
  pub fn modifier(&self) -> Modifier {
    self.modifier
  }

  #[inline]
  pub fn set_modifier(self, modifier: Modifier) -> Option<Self> {
    Some(Self {
      departments: self.departments,
      modifier,
    })
  }
}

impl fmt::Display for OperationCreate {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}{}", OperationCreate::keyword(), fmt_list(&self.departments), fmt_modifier(self.modifier))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OperationRemove {
  pub names: Vec<String>,
  pub departments: Vec<String>,
  modifier: Modifier,
}

impl OperationRemove {
  #[inline]
  pub fn new(departments: Vec<String>, names: Vec<String>, fail_silently: bool) -> Self {
    Self {
      departments,
      names,
      modifier: if fail_silently {
        Modifier::FailSilently
      } else {
        Modifier::None
      },
    }
  }

  pub const fn keyword() -> &'static str {
    "Remove"
  }

  #[inline]
  pub fn modifier(&self) -> Modifier {
    self.modifier
  }

  #[inline]
  pub fn set_modifier(self, modifier: Modifier) -> Option<Self> {
    let new = match modifier {
      Modifier::FailSilently | Modifier::None => Some(modifier),
      _ => None,
    };
    new.map(|modifier| Self {
      departments: self.departments,
      names: self.names,
      modifier,
    })
  }
}

impl fmt::Display for OperationRemove {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let names = fmt_list(&self.names);
    let names = if names.is_empty() {
      "".into()
    } else {
      format!(" {} from", names)
    };
    write!(f, "{}{} {}{}", OperationRemove::keyword(), names, fmt_list(&self.departments), fmt_modifier(self.modifier))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OperationShow {
  pub departments: Vec<String>,
  modifier: Modifier,
}

impl OperationShow {
  #[inline]
  pub fn new(departments: Vec<String>, fail_silently: bool) -> Self {
    Self {
      departments,
      modifier: if fail_silently {
        Modifier::FailSilently
      } else {
        Modifier::None
      },
    }
  }

  pub const fn keyword() -> &'static str {
    "Show"
  }

  #[inline]
  pub fn modifier(&self) -> Modifier {
    self.modifier
  }

  #[inline]
  pub fn set_modifier(self, modifier: Modifier) -> Option<Self> {
    let new = match modifier {
      Modifier::FailSilently | Modifier::None => Some(modifier),
      _ => None,
    };
    new.map(|modifier| Self {
      departments: self.departments,
      modifier,
    })
  }
}

impl fmt::Display for OperationShow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}{}", OperationShow::keyword(), fmt_list(&self.departments), fmt_modifier(self.modifier))
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Operation {
  Add(OperationAdd),
  Create(OperationCreate),
  Remove(OperationRemove),
  Show(OperationShow),
  Unknown,
}

impl Operation {
  pub fn modifier(&self) -> Modifier {
    use Operation::*;

    match self {
      Add(op) => op.modifier(),
      Create(op) => op.modifier(),
      Remove(op) => op.modifier(),
      Show(op) => op.modifier(),
      Unknown => Modifier::None,
    }
  }

  pub fn set_modifier(self, modifier: Modifier) -> Option<Self> {
    use Operation::*;

    match self {
      Add(op) => op.set_modifier(modifier).map(Into::into),
      Create(op) => op.set_modifier(modifier).map(Into::into),
      Remove(op) => op.set_modifier(modifier).map(Into::into),
      Show(op) => op.set_modifier(modifier).map(Into::into),
      Unknown => None,
    }
  }

  pub const fn keyword(&self) -> &'static str {
    use Operation::*;

    match self {
      Add(_) => OperationAdd::keyword(),
      Create(_) => OperationCreate::keyword(),
      Remove(_) => OperationRemove::keyword(),
      Show(_) => OperationShow::keyword(),
      Unknown => "Unknown",
    }
  }
}

impl From<OperationAdd> for Operation {
  fn from(op: OperationAdd) -> Self {
    Operation::Add(op)
  }
}

impl From<OperationCreate> for Operation {
  fn from(op: OperationCreate) -> Self {
    Operation::Create(op)
  }
}

impl From<OperationRemove> for Operation {
  fn from(op: OperationRemove) -> Self {
    Operation::Remove(op)
  }
}

impl From<OperationShow> for Operation {
  fn from(op: OperationShow) -> Self {
    Operation::Show(op)
  }
}

impl fmt::Debug for Operation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Operation::*;
    use fmt::Debug;

    match self {
      Unknown => f.write_str(self.keyword()),
      Add(op) => Debug::fmt(op, f),
      Create(op) => Debug::fmt(op, f),
      Remove(op) => Debug::fmt(op, f),
      Show(op) => Debug::fmt(op, f),
    }
  }
}

impl fmt::Display for Operation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Operation::*;
    use fmt::Display;

    match self {
      Unknown => f.write_str(self.keyword()),
      Add(op) => Display::fmt(op, f),
      Create(op) => Display::fmt(op, f),
      Remove(op) => Display::fmt(op, f),
      Show(op) => Display::fmt(op, f),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn format() {
    let ops = &[
      (Operation::Unknown, "Unknown"),
      (
        OperationAdd::new(
          util::to_string_vec(vec!["Science", "Engineering"]),
          vec!["Mama".into(), "Tata".into(), "Bunica Miha".into()],
          Modifier::None,
        )
        .into(),
        "Add Mama, Tata, and Bunica Miha to Science and Engineering",
      ),
      (
        OperationRemove::new(
          util::to_string_vec(vec!["Engineering"]),
          vec!["Sally".into()],
          true,
        )
        .into(),
        "Remove Sally from Engineering (fail silently)",
      ),
      (
        OperationCreate::new(util::to_string_vec(vec!["Sales"]), Modifier::Overwrite).into(),
        "Create Sales (overwrite if existing)",
      ),
      (
        OperationShow::new(util::to_string_vec(vec!["HR"]), false).into(),
        "Show HR",
      ),
    ];

    ops
      .iter()
      .for_each(|(op, expect)| assert_eq!(format!("{}", op), *expect));
  }
}
