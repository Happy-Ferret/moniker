use {Bound, BoundPattern, BoundTerm, ScopeState};

/// Embed a term in a pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Embed<T>(pub T);

impl<T: BoundTerm> BoundPattern for Embed<T> {
    type Free = T::Free;

    fn pattern_eq(&self, other: &Embed<T>) -> bool {
        T::term_eq(&self.0, &other.0)
    }

    fn freshen(&mut self) -> Vec<T::Free> {
        Vec::new()
    }

    fn rename(&mut self, _perm: &[T::Free]) {}

    fn close_pattern<P>(&mut self, state: ScopeState, pattern: &P)
    where
        P: BoundPattern<Free = Self::Free>,
    {
        self.0.close_term(state, pattern);
    }

    fn open_pattern<P>(&mut self, state: ScopeState, pattern: &P)
    where
        P: BoundPattern<Free = Self::Free>,
    {
        self.0.open_term(state, pattern);
    }

    fn on_free(&self, _state: ScopeState, _name: &Self::Free) -> Option<Bound> {
        None
    }

    fn on_bound(&self, _state: ScopeState, _name: Bound) -> Option<Self::Free> {
        None
    }
}
