use std::collections::HashSet;
use std::rc::Rc;

use bound_pattern::BoundPattern;
use var::{DebruijnIndex, FreeVar, Var};

#[derive(Debug, Copy, Clone)]
pub struct ScopeState {
    depth: u32,
}

impl ScopeState {
    pub fn new() -> ScopeState {
        ScopeState { depth: 0 }
    }

    pub fn depth(&self) -> DebruijnIndex {
        DebruijnIndex(self.depth)
    }

    pub fn incr(mut self) -> ScopeState {
        self.depth += 1;
        self
    }
}

pub trait BoundTerm {
    /// Alpha equivalence in a term context
    fn term_eq(&self, other: &Self) -> bool;

    #[allow(unused_variables)]
    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {}

    #[allow(unused_variables)]
    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {}

    #[allow(unused_variables)]
    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {}

    #[allow(unused_variables)]
    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {}

    fn free_vars(&self) -> HashSet<FreeVar> {
        let mut free_vars = HashSet::new();
        self.visit_vars(&mut |var| match *var {
            Var::Bound(_, _) => {},
            Var::Free(ref name) => {
                free_vars.insert(name.clone());
            },
        });
        free_vars
    }
}

/// Asserts that two expressions are alpha equivalent to each other (using
/// `BoundTerm::term_eq`).
///
/// On panic, this macro will print the values of the expressions with their
/// debug representations.
///
/// Like `assert!`, this macro has a second form, where a custom
/// panic message can be provided.
#[macro_export]
macro_rules! assert_term_eq {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !::moniker::BoundTerm::term_eq(left_val, right_val) {
                    panic!(r#"assertion failed: `<_>::term_eq(&left, &right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr,) => ({
        assert_term_eq!($left, $right)
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !::moniker::BoundTerm::term_eq(left_val, right_val) {
                    panic!(r#"assertion failed: `<_>::term_eq(&left, &right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, left_val, right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
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

// Implementations for common types

macro_rules! impl_bound_term {
    ($T:ty) => {
        impl BoundTerm for $T {
            fn term_eq(&self, other: &$T) -> bool {
                self == other
            }
        }
    };
}

impl_bound_term!(());
impl_bound_term!(String);
impl_bound_term!(str);
impl_bound_term!(char);
impl_bound_term!(bool);
impl_bound_term!(u8);
impl_bound_term!(u16);
impl_bound_term!(u32);
impl_bound_term!(u64);
impl_bound_term!(usize);
impl_bound_term!(i8);
impl_bound_term!(i16);
impl_bound_term!(i32);
impl_bound_term!(i64);
impl_bound_term!(isize);
impl_bound_term!(f32);
impl_bound_term!(f64);

impl<T: BoundTerm> BoundTerm for Option<T> {
    fn term_eq(&self, other: &Option<T>) -> bool {
        match (self, other) {
            (&Some(ref lhs), &Some(ref rhs)) => T::term_eq(lhs, rhs),
            (_, _) => false,
        }
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        if let Some(ref mut inner) = *self {
            inner.close_term(state, pattern);
        }
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        if let Some(ref mut inner) = *self {
            inner.open_term(state, pattern);
        }
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        if let Some(ref inner) = *self {
            inner.visit_vars(on_var);
        }
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        if let Some(ref mut inner) = *self {
            inner.visit_mut_vars(on_var);
        }
    }
}

impl<T: BoundTerm> BoundTerm for Box<T> {
    fn term_eq(&self, other: &Box<T>) -> bool {
        T::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        (**self).close_term(state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        (**self).open_term(state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        (**self).visit_vars(on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        (**self).visit_mut_vars(on_var);
    }
}

impl<T: BoundTerm + Clone> BoundTerm for Rc<T> {
    fn term_eq(&self, other: &Rc<T>) -> bool {
        T::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        Rc::make_mut(self).close_term(state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        Rc::make_mut(self).open_term(state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        (**self).visit_vars(on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        Rc::make_mut(self).visit_mut_vars(on_var);
    }
}

impl<T: BoundTerm + Clone> BoundTerm for [T] {
    fn term_eq(&self, other: &[T]) -> bool {
        self.len() == other.len()
            && <_>::zip(self.iter(), other.iter()).all(|(lhs, rhs)| T::term_eq(lhs, rhs))
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        for elem in self {
            elem.close_term(state, pattern);
        }
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        for elem in self {
            elem.open_term(state, pattern);
        }
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        for elem in self {
            elem.visit_vars(on_var);
        }
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        for elem in self {
            elem.visit_mut_vars(on_var);
        }
    }
}

impl<T: BoundTerm + Clone> BoundTerm for Vec<T> {
    fn term_eq(&self, other: &Vec<T>) -> bool {
        <[T]>::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        <[T]>::close_term(self, state, pattern)
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern) {
        <[T]>::open_term(self, state, pattern)
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var)) {
        <[T]>::visit_vars(self, on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var)) {
        <[T]>::visit_mut_vars(self, on_var);
    }
}
