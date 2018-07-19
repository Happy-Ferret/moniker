use bound::{BoundPattern, Permutations, ScopeState};
use var::{FreeVar, PVar, PVarIndex, PVarOffset};

/// Nested binding patterns
///
/// Contrast with `Multi`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nest<P> {
    pub unsafe_patterns: Vec<P>,
}

impl<P> Nest<P> {
    /// Nest a term with the given patterns
    pub fn new<Ident>(patterns: Vec<P>) -> Nest<P>
    where
        P: BoundPattern<Ident>,
    {
        // FIXME: Avoid allocating new vector
        let mut rebound_patterns = Vec::<P>::with_capacity(patterns.len());

        for mut pattern in patterns {
            let mut state = ScopeState::new();
            for rebound_pattern in &rebound_patterns {
                state = state.incr();
                pattern.close_pattern(state, rebound_pattern);
            }
            rebound_patterns.push(pattern);
        }

        Nest {
            unsafe_patterns: rebound_patterns,
        }
    }

    /// Unnest a term, returning the freshened patterns
    pub fn unnest<Ident>(self) -> Vec<P>
    where
        P: BoundPattern<Ident>,
    {
        // FIXME: Avoid allocating new vector
        let mut unrebound_patterns = Vec::<P>::with_capacity(self.unsafe_patterns.len());

        for mut pattern in self.unsafe_patterns {
            let mut state = ScopeState::new();
            for bound_pattern in &unrebound_patterns {
                state = state.incr();
                pattern.open_pattern(state, bound_pattern);
            }
            unrebound_patterns.push(pattern);
        }

        unrebound_patterns
    }
}

impl<Ident, P> BoundPattern<Ident> for Nest<P>
where
    Ident: Clone,
    P: BoundPattern<Ident>,
{
    fn pattern_eq(&self, other: &Nest<P>) -> bool {
        <[P]>::pattern_eq(&self.unsafe_patterns, &other.unsafe_patterns)
    }

    fn freshen(&mut self, permutations: &mut Permutations<Ident>) {
        <[P]>::freshen(&mut self.unsafe_patterns, permutations)
    }

    fn swaps(&mut self, permutations: &Permutations<Ident>) {
        <[P]>::swaps(&mut self.unsafe_patterns, permutations)
    }

    fn close_pattern(&mut self, mut state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in &mut self.unsafe_patterns {
            elem.close_pattern(state, pattern);
            state = state.incr();
        }
    }

    fn open_pattern(&mut self, mut state: ScopeState, pattern: &impl BoundPattern<Ident>) {
        for elem in &mut self.unsafe_patterns {
            elem.close_pattern(state, pattern);
            state = state.incr();
        }
    }

    fn find_pvar_index(&self, free_var: &FreeVar<Ident>) -> Result<PVarIndex, PVarOffset> {
        <[P]>::find_pvar_index(&self.unsafe_patterns, free_var)
    }

    fn find_pvar_at_offset(&self, offset: PVarOffset) -> Result<PVar<Ident>, PVarOffset> {
        <[P]>::find_pvar_at_offset(&self.unsafe_patterns, offset)
    }
}
