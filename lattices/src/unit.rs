use super::Merge;
use crate::{IsBot, IsTop, LatticeFrom, LatticeOrd};

/// A `Unit` lattice that contains no state
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Unit;
impl Unit {
    /// Create a new `Unit` lattice instance from a value.
    pub fn new() -> Self {
        Self
    }
}

impl Merge<Unit> for Unit {
    fn merge(&mut self, _: Unit) -> bool {
        false
    }
}

impl LatticeFrom<Unit> for Unit {
    fn lattice_from(other: Unit) -> Self {
        other
    }
}

impl LatticeOrd<Unit> for Unit {}

impl IsBot for Unit {
    fn is_bot(&self) -> bool {
        true
    }
}

impl IsTop for Unit {
    fn is_top(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::check_all;

    #[test]
    fn consistency() {
        check_all(&[Unit::new()])
    }
}
