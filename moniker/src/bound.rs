#[cfg(feature = "codespan")]
use codespan::{
    ByteIndex, ByteOffset, ColumnIndex, ColumnNumber, ColumnOffset, LineIndex, LineNumber,
    LineOffset, Span,
};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::{slice, vec};

use var::{BoundVar, FreeVar, GenId, PatternIndex, ScopeOffset, Var};

#[derive(Debug, Copy, Clone)]
pub struct ScopeState {
    depth: u32,
}

impl ScopeState {
    pub fn new() -> ScopeState {
        ScopeState { depth: 0 }
    }

    pub fn depth(&self) -> ScopeOffset {
        ScopeOffset(self.depth)
    }

    pub fn incr(mut self) -> ScopeState {
        self.depth += 1;
        self
    }
}

pub trait BoundTerm<Ident> {
    /// Alpha equivalence in a term context
    fn term_eq(&self, other: &Self) -> bool;

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>);

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>);

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>));

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>));

    fn free_vars(&self) -> HashSet<FreeVar<Ident>>
    where
        Ident: Eq + Hash + Clone,
    {
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

impl<Ident: PartialEq> BoundTerm<Ident> for FreeVar<Ident> {
    fn term_eq(&self, other: &FreeVar<Ident>) -> bool {
        match (self, other) {
            (&FreeVar::User(ref lhs), &FreeVar::User(ref rhs)) => lhs == rhs,
            (&FreeVar::Gen(ref lhs, _), &FreeVar::Gen(ref rhs, _)) => lhs == rhs,
            _ => false,
        }
    }

    fn close_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn open_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn visit_vars(&self, _: &mut impl FnMut(&Var<Ident>)) {}

    fn visit_mut_vars(&mut self, _: &mut impl FnMut(&mut Var<Ident>)) {}
}

impl<Ident: PartialEq + Clone> BoundTerm<Ident> for Var<Ident> {
    fn term_eq(&self, other: &Var<Ident>) -> bool {
        match (self, other) {
            (&Var::Free(ref lhs), &Var::Free(ref rhs)) => FreeVar::term_eq(lhs, rhs),
            (&Var::Bound(ref lhs, _), &Var::Bound(ref rhs, _)) => lhs == rhs,
            (_, _) => false,
        }
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        *self = match *self {
            Var::Bound(_, _) => return,
            Var::Free(ref name) => match pattern.on_free(state, name) {
                Some(bound) => Var::Bound(bound, name.ident().cloned()),
                None => return,
            },
        };
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        *self = match *self {
            Var::Free(_) => return,
            Var::Bound(bound, _) => match pattern.on_bound(state, bound) {
                Some(name) => Var::Free(name),
                None => return,
            },
        };
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        on_var(self);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        on_var(self);
    }
}

// Implementations for common types

macro_rules! impl_bound_term_partial_eq {
    ($T:ty) => {
        impl<Ident> BoundTerm<Ident> for $T {
            fn term_eq(&self, other: &$T) -> bool {
                self == other
            }

            fn close_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn open_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn visit_vars(&self, _: &mut impl FnMut(&Var<Ident>)) {}

            fn visit_mut_vars(&mut self, _: &mut impl FnMut(&mut Var<Ident>)) {}
        }
    };
}

impl_bound_term_partial_eq!(());
impl_bound_term_partial_eq!(String);
impl_bound_term_partial_eq!(str);
impl_bound_term_partial_eq!(char);
impl_bound_term_partial_eq!(bool);
impl_bound_term_partial_eq!(u8);
impl_bound_term_partial_eq!(u16);
impl_bound_term_partial_eq!(u32);
impl_bound_term_partial_eq!(u64);
impl_bound_term_partial_eq!(usize);
impl_bound_term_partial_eq!(i8);
impl_bound_term_partial_eq!(i16);
impl_bound_term_partial_eq!(i32);
impl_bound_term_partial_eq!(i64);
impl_bound_term_partial_eq!(isize);
impl_bound_term_partial_eq!(f32);
impl_bound_term_partial_eq!(f64);

#[cfg(feature = "codespan")]
macro_rules! impl_bound_term_ignore {
    ($T:ty) => {
        impl<Ident> BoundTerm<Ident> for $T {
            fn term_eq(&self, _: &$T) -> bool {
                true
            }

            fn close_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn open_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn visit_vars(&self, _: &mut impl FnMut(&Var<Ident>)) {}

            fn visit_mut_vars(&mut self, _: &mut impl FnMut(&mut Var<Ident>)) {}
        }
    };
}

#[cfg(feature = "codespan")]
impl_bound_term_ignore!(ByteIndex);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(ByteOffset);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(ColumnIndex);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(ColumnNumber);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(ColumnOffset);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(LineIndex);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(LineNumber);
#[cfg(feature = "codespan")]
impl_bound_term_ignore!(LineOffset);

#[cfg(feature = "codespan")]
impl<Ident, T> BoundTerm<Ident> for Span<T> {
    fn term_eq(&self, _: &Span<T>) -> bool {
        true
    }

    fn close_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn open_term(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn visit_vars(&self, _: &mut impl FnMut(&Var<Ident>)) {}

    fn visit_mut_vars(&mut self, _: &mut impl FnMut(&mut Var<Ident>)) {}
}

impl<Ident, T> BoundTerm<Ident> for Option<T>
where
    T: BoundTerm<Ident>,
{
    fn term_eq(&self, other: &Option<T>) -> bool {
        match (self, other) {
            (&Some(ref lhs), &Some(ref rhs)) => T::term_eq(lhs, rhs),
            (&None, &None) => true,
            (_, _) => false,
        }
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        if let Some(ref mut inner) = *self {
            inner.close_term(state, pattern);
        }
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        if let Some(ref mut inner) = *self {
            inner.open_term(state, pattern);
        }
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        if let Some(ref inner) = *self {
            inner.visit_vars(on_var);
        }
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        if let Some(ref mut inner) = *self {
            inner.visit_mut_vars(on_var);
        }
    }
}

impl<Ident, T> BoundTerm<Ident> for Box<T>
where
    T: BoundTerm<Ident>,
{
    fn term_eq(&self, other: &Box<T>) -> bool {
        T::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        T::close_term(self, state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        T::open_term(self, state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        T::visit_vars(self, on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        T::visit_mut_vars(self, on_var);
    }
}

impl<Ident, T> BoundTerm<Ident> for Rc<T>
where
    T: BoundTerm<Ident> + Clone,
{
    fn term_eq(&self, other: &Rc<T>) -> bool {
        T::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        T::close_term(Rc::make_mut(self), state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        T::open_term(Rc::make_mut(self), state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        T::visit_vars(self, on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        T::visit_mut_vars(Rc::make_mut(self), on_var);
    }
}

impl<Ident, T, U> BoundTerm<Ident> for (T, U)
where
    T: BoundTerm<Ident>,
    U: BoundTerm<Ident>,
{
    fn term_eq(&self, other: &(T, U)) -> bool {
        T::term_eq(&self.0, &other.0) && U::term_eq(&self.1, &other.1)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.close_term(state, pattern);
        self.1.close_term(state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.open_term(state, pattern);
        self.1.open_term(state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        self.0.visit_vars(on_var);
        self.1.visit_vars(on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        self.0.visit_mut_vars(on_var);
        self.1.visit_mut_vars(on_var);
    }
}

impl<Ident, T, U, V> BoundTerm<Ident> for (T, U, V)
where
    T: BoundTerm<Ident>,
    U: BoundTerm<Ident>,
    V: BoundTerm<Ident>,
{
    fn term_eq(&self, other: &(T, U, V)) -> bool {
        T::term_eq(&self.0, &other.0)
            && U::term_eq(&self.1, &other.1)
            && V::term_eq(&self.2, &other.2)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.close_term(state, pattern);
        self.1.close_term(state, pattern);
        self.2.close_term(state, pattern);
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.open_term(state, pattern);
        self.1.open_term(state, pattern);
        self.2.open_term(state, pattern);
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        self.0.visit_vars(on_var);
        self.1.visit_vars(on_var);
        self.2.visit_vars(on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        self.0.visit_mut_vars(on_var);
        self.1.visit_mut_vars(on_var);
        self.2.visit_mut_vars(on_var);
    }
}

impl<Ident, T> BoundTerm<Ident> for [T]
where
    T: BoundTerm<Ident> + Clone,
{
    fn term_eq(&self, other: &[T]) -> bool {
        self.len() == other.len()
            && <_>::zip(self.iter(), other.iter()).all(|(lhs, rhs)| T::term_eq(lhs, rhs))
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in self {
            elem.close_term(state, pattern);
        }
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in self {
            elem.open_term(state, pattern);
        }
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        for elem in self {
            elem.visit_vars(on_var);
        }
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        for elem in self {
            elem.visit_mut_vars(on_var);
        }
    }
}

impl<Ident, T> BoundTerm<Ident> for Vec<T>
where
    T: BoundTerm<Ident> + Clone,
{
    fn term_eq(&self, other: &Vec<T>) -> bool {
        <[T]>::term_eq(self, other)
    }

    fn close_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        <[T]>::close_term(self, state, pattern)
    }

    fn open_term(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        <[T]>::open_term(self, state, pattern)
    }

    fn visit_vars(&self, on_var: &mut impl FnMut(&Var<Ident>)) {
        <[T]>::visit_vars(self, on_var);
    }

    fn visit_mut_vars(&mut self, on_var: &mut impl FnMut(&mut Var<Ident>)) {
        <[T]>::visit_mut_vars(self, on_var);
    }
}

/// A mapping of `PatternIndex`s to `T`s
pub struct PatternSubsts<T> {
    permutations: Vec<T>,
}

impl<T> PatternSubsts<T> {
    pub fn new(permutations: Vec<T>) -> PatternSubsts<T> {
        PatternSubsts { permutations }
    }

    pub fn lookup(&self, index: PatternIndex) -> Option<&T> {
        self.permutations.get(index.0 as usize)
    }

    pub fn push(&mut self, value: T) {
        self.permutations.push(value);
    }

    pub fn len(&self) -> usize {
        self.permutations.len()
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.permutations.iter()
    }
}

impl<T> Extend<T> for PatternSubsts<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.permutations.extend(iter)
    }
}

impl<T> IntoIterator for PatternSubsts<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> vec::IntoIter<T> {
        self.permutations.into_iter()
    }
}

pub trait BoundPattern<Ident> {
    /// Alpha equivalence in a pattern context
    fn pattern_eq(&self, other: &Self) -> bool;

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>);

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>);

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>);

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>);

    /// A callback that is used when `unbind`ing `Bind`s to replace free names
    /// with bound names based on the contents of the pattern
    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar>;

    /// A callback that is used when `bind`ing `Bind`s to replace bound names
    /// with free names based on the contents of the pattern
    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>>;
}

impl<Ident: Clone + PartialEq> BoundPattern<Ident> for FreeVar<Ident> {
    fn pattern_eq(&self, _other: &FreeVar<Ident>) -> bool {
        true
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        *self = match *self {
            FreeVar::User(ref name) => FreeVar::Gen(GenId::fresh(), Some(name.clone())),
            FreeVar::Gen(_, _) => {
                permutations.push(self.clone());
                return;
            },
        };
        permutations.push(self.clone());
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        assert_eq!(permutations.len(), 1); // FIXME: assert
        *self = permutations.lookup(PatternIndex(0)).unwrap().clone(); // FIXME: double clone
    }

    fn close_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn open_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        match FreeVar::term_eq(name, self) {
            true => Some(BoundVar {
                scope: state.depth(),
                pattern: PatternIndex(0),
            }),
            false => None,
        }
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        match name.scope == state.depth() {
            true => {
                assert_eq!(name.pattern, PatternIndex(0));
                Some(self.clone())
            },
            false => None,
        }
    }
}

// Implementations for common types

macro_rules! impl_bound_pattern_partial_eq {
    ($T:ty) => {
        impl<Ident> BoundPattern<Ident> for $T {
            fn pattern_eq(&self, other: &$T) -> bool {
                self == other
            }

            fn freshen(&mut self, _: &mut PatternSubsts<FreeVar<Ident>>) {}

            fn swaps(&mut self, _: &PatternSubsts<FreeVar<Ident>>) {}

            fn close_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn open_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn on_free(&self, _: ScopeState, _: &FreeVar<Ident>) -> Option<BoundVar> {
                None
            }

            fn on_bound(&self, _: ScopeState, _: BoundVar) -> Option<FreeVar<Ident>> {
                None
            }
        }
    };
}

impl_bound_pattern_partial_eq!(());
impl_bound_pattern_partial_eq!(String);
impl_bound_pattern_partial_eq!(str);
impl_bound_pattern_partial_eq!(char);
impl_bound_pattern_partial_eq!(bool);
impl_bound_pattern_partial_eq!(u8);
impl_bound_pattern_partial_eq!(u16);
impl_bound_pattern_partial_eq!(u32);
impl_bound_pattern_partial_eq!(u64);
impl_bound_pattern_partial_eq!(usize);
impl_bound_pattern_partial_eq!(i8);
impl_bound_pattern_partial_eq!(i16);
impl_bound_pattern_partial_eq!(i32);
impl_bound_pattern_partial_eq!(i64);
impl_bound_pattern_partial_eq!(isize);
impl_bound_pattern_partial_eq!(f32);
impl_bound_pattern_partial_eq!(f64);

#[cfg(feature = "codespan")]
macro_rules! impl_bound_pattern_ignore {
    ($T:ty) => {
        impl<Ident> BoundPattern<Ident> for $T {
            fn pattern_eq(&self, _: &$T) -> bool {
                true
            }

            fn freshen(&mut self, _: &mut PatternSubsts<FreeVar<Ident>>) {}

            fn swaps(&mut self, _: &PatternSubsts<FreeVar<Ident>>) {}

            fn close_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn open_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

            fn on_free(&self, _: ScopeState, _: &FreeVar<Ident>) -> Option<BoundVar> {
                None
            }

            fn on_bound(&self, _: ScopeState, _: BoundVar) -> Option<FreeVar<Ident>> {
                None
            }
        }
    };
}

#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(ByteIndex);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(ByteOffset);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(ColumnIndex);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(ColumnNumber);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(ColumnOffset);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(LineIndex);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(LineNumber);
#[cfg(feature = "codespan")]
impl_bound_pattern_ignore!(LineOffset);

#[cfg(feature = "codespan")]
impl<Ident, T> BoundPattern<Ident> for Span<T> {
    fn pattern_eq(&self, _: &Span<T>) -> bool {
        true
    }

    fn freshen(&mut self, _: &mut PatternSubsts<FreeVar<Ident>>) {}

    fn swaps(&mut self, _: &PatternSubsts<FreeVar<Ident>>) {}

    fn close_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn open_pattern(&mut self, _: ScopeState, _: &impl BoundPattern<Ident>) {}

    fn on_free(&self, _: ScopeState, _: &FreeVar<Ident>) -> Option<BoundVar> {
        None
    }

    fn on_bound(&self, _: ScopeState, _: BoundVar) -> Option<FreeVar<Ident>> {
        None
    }
}

impl<Ident, P> BoundPattern<Ident> for Option<P>
where
    P: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &Option<P>) -> bool {
        match (self, other) {
            (&Some(ref lhs), &Some(ref rhs)) => P::pattern_eq(lhs, rhs),
            (&None, &None) => true,
            (_, _) => false,
        }
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        if let Some(ref mut inner) = *self {
            inner.freshen(permutations);
        }
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        if let Some(ref mut inner) = *self {
            inner.swaps(permutations);
        }
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        if let Some(ref mut inner) = *self {
            inner.close_pattern(state, pattern);
        }
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        if let Some(ref mut inner) = *self {
            inner.open_pattern(state, pattern);
        }
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        self.as_ref().and_then(|inner| inner.on_free(state, name))
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        self.as_ref().and_then(|inner| inner.on_bound(state, name))
    }
}

impl<Ident, P1, P2> BoundPattern<Ident> for (P1, P2)
where
    P1: BoundPattern<Ident>,
    P2: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &(P1, P2)) -> bool {
        P1::pattern_eq(&self.0, &other.0) && P2::pattern_eq(&self.1, &other.1)
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        self.0.freshen(permutations);
        self.1.freshen(permutations);
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        self.0.swaps(permutations);
        self.1.swaps(permutations);
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.close_pattern(state, pattern);
        self.1.close_pattern(state, pattern);
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        self.0.open_pattern(state, pattern);
        self.1.open_pattern(state, pattern);
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        self.0
            .on_free(state, name)
            .or_else(|| self.1.on_free(state, name))
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        self.0
            .on_bound(state, name)
            .or_else(|| self.1.on_bound(state, name))
    }
}

impl<Ident, P> BoundPattern<Ident> for Box<P>
where
    P: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &Box<P>) -> bool {
        P::pattern_eq(self, other)
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        P::freshen(self, permutations)
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        P::swaps(self, permutations);
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        P::close_pattern(self, state, pattern);
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        P::open_pattern(self, state, pattern);
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        P::on_free(self, state, name)
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        P::on_bound(self, state, name)
    }
}

impl<Ident, P> BoundPattern<Ident> for Rc<P>
where
    P: BoundPattern<Ident> + Clone,
{
    fn pattern_eq(&self, other: &Rc<P>) -> bool {
        P::pattern_eq(self, other)
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        P::freshen(Rc::make_mut(self), permutations)
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        P::swaps(Rc::make_mut(self), permutations);
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        P::close_pattern(Rc::make_mut(self), state, pattern);
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        P::open_pattern(Rc::make_mut(self), state, pattern);
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        P::on_free(self, state, name)
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        P::on_bound(self, state, name)
    }
}

impl<Ident, P> BoundPattern<Ident> for [P]
where
    Ident: Clone,
    P: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &[P]) -> bool {
        self.len() == other.len()
            && <_>::zip(self.iter(), other.iter()).all(|(lhs, rhs)| P::pattern_eq(lhs, rhs))
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        for elem in self {
            elem.freshen(permutations);
        }
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        assert_eq!(self.len(), permutations.len()); // FIXME: assertion

        for (pattern, free_var) in <_>::zip(self.iter_mut(), permutations.iter()) {
            pattern.swaps(&PatternSubsts::new(vec![free_var.clone()])); // FIXME: clone
        }
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in self {
            elem.close_pattern(state, pattern);
        }
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in self {
            elem.open_pattern(state, pattern);
        }
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        self.iter()
            .enumerate()
            .filter_map(|(i, pattern)| {
                pattern.on_free(state, name).map(|bound| {
                    assert_eq!(bound.pattern, PatternIndex(0));
                    BoundVar {
                        pattern: PatternIndex(i as u32),
                        ..bound
                    }
                })
            })
            .next()
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        self.get(name.pattern.0 as usize).and_then(|pattern| {
            pattern.on_bound(
                state,
                BoundVar {
                    pattern: PatternIndex(0),
                    ..name
                },
            )
        })
    }
}

impl<Ident, P> BoundPattern<Ident> for Vec<P>
where
    Ident: Clone,
    P: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &Vec<P>) -> bool {
        <[P]>::pattern_eq(self, other)
    }

    fn freshen(&mut self, permutations: &mut PatternSubsts<FreeVar<Ident>>) {
        <[P]>::freshen(self, permutations)
    }

    fn swaps(&mut self, permutations: &PatternSubsts<FreeVar<Ident>>) {
        <[P]>::swaps(self, permutations);
    }

    fn close_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        <[P]>::close_pattern(self, state, pattern);
    }

    fn open_pattern(&mut self, state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        <[P]>::open_pattern(self, state, pattern);
    }

    fn on_free(&self, state: ScopeState, name: &FreeVar<Ident>) -> Option<BoundVar> {
        <[P]>::on_free(self, state, name)
    }

    fn on_bound(&self, state: ScopeState, name: BoundVar) -> Option<FreeVar<Ident>> {
        <[P]>::on_bound(self, state, name)
    }
}
