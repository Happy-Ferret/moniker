use std::fmt;

use {BoundPattern, BoundTerm, Name, ScopeState};

/// The [Debruijn index] of the binder that introduced the variable
///
/// For example:
///
/// ```text
/// λx.∀y.λz. x z (y z)
/// λ  ∀  λ   2 0 (1 0)
/// ```
///
/// [Debruijn index]: https://en.wikipedia.org/wiki/De_Bruijn_index
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Debruijn(pub u32);

impl Debruijn {
    /// Move the current Debruijn index into an inner binder
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PatternIndex(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bound {
    pub scope: Debruijn,
    pub pattern: PatternIndex,
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.scope.0, self.pattern.0)
    }
}

/// A variable that can either be free or bound
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Var {
    /// A free variable
    Free(Name),
    /// A variable that is bound by a lambda or pi binder
    Bound(Name, Bound),
}

impl BoundTerm for Var {
    fn term_eq(&self, other: &Var) -> bool {
        match (self, other) {
            (&Var::Free(ref lhs), &Var::Free(ref rhs)) => Name::term_eq(lhs, rhs),
            (&Var::Bound(_, ref lhs), &Var::Bound(_, ref rhs)) => lhs == rhs,
            (_, _) => false,
        }
    }

    fn close_term<P: BoundPattern>(&mut self, state: ScopeState, pattern: &P) {
        *self = match *self {
            Var::Bound(_, _) => return,
            Var::Free(ref name) => match pattern.on_free(state, name) {
                Some(bound) => Var::Bound(name.clone(), bound),
                None => return,
            },
        };
    }

    fn open_term<P: BoundPattern>(&mut self, state: ScopeState, pattern: &P) {
        *self = match *self {
            Var::Free(_) => return,
            Var::Bound(_, bound) => match pattern.on_bound(state, bound) {
                Some(name) => Var::Free(name),
                None => return,
            },
        };
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Var::Bound(ref name, bound) if f.alternate() => write!(f, "{}@{}", name, bound),
            Var::Bound(ref name, ..) | Var::Free(ref name) => write!(f, "{}", name),
        }
    }
}
