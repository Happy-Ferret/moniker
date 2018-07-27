use std::fmt;

use free_var::FreeVar;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BinderIndex(pub u32);

impl BinderIndex {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for BinderIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binder<N>(pub FreeVar<N>);

impl<N> Binder<N> {
    /// Create a variable from a human-readable string
    pub fn user<T: Into<N>>(ident: T) -> Binder<N> {
        Binder(FreeVar::user(ident))
    }

    pub fn freshen(self) -> Binder<N> {
        Binder(self.0.freshen())
    }
}

impl<N: fmt::Display> fmt::Display for Binder<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<N> PartialEq<FreeVar<N>> for Binder<N>
where
    N: PartialEq,
{
    fn eq(&self, other: &FreeVar<N>) -> bool {
        self.0 == *other
    }
}
