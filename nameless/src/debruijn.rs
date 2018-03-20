use std::fmt;

use AlphaEq;

/// The [debruijn index] of the binder that introduced the variable
///
/// For example:
///
/// ```text
/// λx.∀y.λz. x z (y z)
/// λ  ∀  λ   2 0 (1 0)
/// ```
///
/// [debruijn index]: https://en.wikipedia.org/wiki/De_Bruijn_index
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Debruijn(pub u32);

impl Debruijn {
    /// Move the current debruijn index into an inner binder
    pub fn succ(self) -> Debruijn {
        Debruijn(self.0 + 1)
    }

    pub fn pred(self) -> Option<Debruijn> {
        match self {
            Debruijn(0) => None,
            Debruijn(i) => Some(Debruijn(i - 1)),
        }
    }
}

impl AlphaEq for Debruijn {
    fn alpha_eq(&self, other: &Debruijn) -> bool {
        self == other
    }
}

impl fmt::Display for Debruijn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}
