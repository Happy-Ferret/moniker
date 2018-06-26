use std::fmt;

use {BoundPattern, BoundTerm, PatternSubsts, ScopeState};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident(String);

impl<'a> From<&'a str> for Ident {
    fn from(src: &'a str) -> Ident {
        Ident(String::from(src))
    }
}

impl From<String> for Ident {
    fn from(src: String) -> Ident {
        Ident(src)
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A source of fresh variables
#[derive(Debug)]
pub struct FreshState {
    next_id: u32,
}

impl FreshState {
    /// Construct a new
    pub fn new() -> FreshState {
        FreshState { next_id: 0 }
    }

    /// Generate a fresh id
    pub fn gen(&mut self) -> FreshId {
        let next_id = self.next_id;
        self.next_id += 1;
        FreshId(next_id)
    }
}

/// A fresh id
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FreshId(u32);

impl fmt::Display for FreshId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}

/// A free variable
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FreeVar {
    /// Names originating from user input
    User(Ident),
    /// A generated id with an optional string that may have come from user
    /// input (for debugging purposes)
    Gen(FreshId, Option<Ident>),
}

impl FreeVar {
    /// Create a name from a human-readable string
    pub fn user<S: Into<Ident>>(name: S) -> FreeVar {
        FreeVar::User(name.into())
    }

    pub fn ident(&self) -> Option<&Ident> {
        match *self {
            FreeVar::User(ref name) => Some(name),
            FreeVar::Gen(_, ref hint) => hint.as_ref(),
        }
    }
}

impl BoundTerm for FreeVar {
    fn term_eq(&self, other: &FreeVar) -> bool {
        match (self, other) {
            (&FreeVar::User(ref lhs), &FreeVar::User(ref rhs)) => lhs == rhs,
            (&FreeVar::Gen(ref lhs, _), &FreeVar::Gen(ref rhs, _)) => lhs == rhs,
            _ => false,
        }
    }
}

impl BoundPattern for FreeVar {
    fn pattern_eq(&self, _other: &FreeVar) -> bool {
        true
    }

    fn freshen(&mut self, fresh_state: &mut FreshState) -> PatternSubsts<FreeVar> {
        *self = match *self {
            FreeVar::User(ref name) => FreeVar::Gen(fresh_state.gen(), Some(name.clone())),
            FreeVar::Gen(_, _) => return PatternSubsts::new(vec![self.clone()]),
        };
        PatternSubsts::new(vec![self.clone()])
    }

    fn rename(&mut self, perm: &PatternSubsts<FreeVar>) {
        assert_eq!(perm.len(), 1); // FIXME: assert
        *self = perm.lookup(PatternIndex(0)).unwrap().clone(); // FIXME: double clone
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar) -> Option<BoundVar> {
        match name == self {
            true => Some(BoundVar {
                scope: state.depth(),
                pattern: PatternIndex(0),
            }),
            false => None,
        }
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar> {
        match name.scope == state.depth() {
            true => {
                assert_eq!(name.pattern, PatternIndex(0));
                Some(self.clone())
            },
            false => None,
        }
    }
}

impl From<FreshId> for FreeVar {
    fn from(src: FreshId) -> FreeVar {
        FreeVar::Gen(src, None)
    }
}

impl fmt::Display for FreeVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FreeVar::User(ref name) => write!(f, "{}", name),
            FreeVar::Gen(ref gen_id, ref name_hint) => match *name_hint {
                None => write!(f, "{}", gen_id),
                Some(ref name) => write!(f, "{}{}", name, gen_id),
            },
        }
    }
}

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
pub struct DebruijnIndex(pub u32);

impl DebruijnIndex {
    /// Move the current Debruijn index into an inner binder
    pub fn succ(self) -> DebruijnIndex {
        DebruijnIndex(self.0 + 1)
    }

    pub fn pred(self) -> Option<DebruijnIndex> {
        match self {
            DebruijnIndex(0) => None,
            DebruijnIndex(i) => Some(DebruijnIndex(i - 1)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PatternIndex(pub u32);

/// A bound variable
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BoundVar {
    pub scope: DebruijnIndex,
    pub pattern: PatternIndex,
}

impl fmt::Display for BoundVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.scope.0, self.pattern.0)
    }
}

/// A variable that can either be free or bound
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Var {
    /// A free variable
    Free(FreeVar),
    /// A variable that is bound by a lambda or pi binder
    Bound(BoundVar, Option<Ident>),
}

impl BoundTerm for Var {
    fn term_eq(&self, other: &Var) -> bool {
        match (self, other) {
            (&Var::Free(ref lhs), &Var::Free(ref rhs)) => FreeVar::term_eq(lhs, rhs),
            (&Var::Bound(ref lhs, _), &Var::Bound(ref rhs, _)) => lhs == rhs,
            (_, _) => false,
        }
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        *self = match *self {
            Var::Bound(_, _) => return,
            Var::Free(ref name) => match pattern.on_free(state, name) {
                Some(bound) => Var::Bound(bound, name.ident().cloned()),
                None => return,
            },
        };
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        *self = match *self {
            Var::Free(_) => return,
            Var::Bound(bound, _) => match pattern.on_bound(state, bound) {
                Some(name) => Var::Free(name),
                None => return,
            },
        };
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        on_var(self);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        on_var(self);
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Var::Bound(bound, None) => write!(f, "@{}", bound),
            Var::Bound(bound, Some(ref hint)) => write!(f, "{}@{}", hint, bound),
            Var::Free(ref free) => write!(f, "{}", free),
        }
    }
}
