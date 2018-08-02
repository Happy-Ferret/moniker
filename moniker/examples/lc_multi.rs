//! An example of using the `moniker` library to implement the untyped lambda
//! calculus with multi-binders (like you would find in Rust, Python, JS, etc.)

#[macro_use]
extern crate moniker;

use moniker::{Binder, Scope, Var};
use std::rc::Rc;

/// Expressions
///
/// ```text
/// e ::= x                     variables
///     | \(x₁, ..., xₙ) => e   anonymous functions
///     | e (e₁, ..., eₙ)       function application
/// ````
#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    /// Variables
    Var(Var<String>),
    /// Lambda expressions
    Lam(Scope<Vec<Binder<String>>, RcExpr>),
    /// Function application
    App(RcExpr, Vec<RcExpr>),
}

/// Reference counted expressions
#[derive(Debug, Clone, BoundTerm)]
pub struct RcExpr {
    pub inner: Rc<Expr>,
}

impl From<Expr> for RcExpr {
    fn from(src: Expr) -> RcExpr {
        RcExpr {
            inner: Rc::new(src),
        }
    }
}

impl RcExpr {
    // FIXME: auto-derive this somehow!
    fn substs<N: PartialEq<Var<String>>>(&self, mappings: &[(N, RcExpr)]) -> RcExpr {
        match *self.inner {
            Expr::Var(ref var) => match mappings.iter().find(|&(name, _)| name == var) {
                Some((_, ref replacement)) => replacement.clone(),
                None => self.clone(),
            },
            Expr::Lam(ref scope) => RcExpr::from(Expr::Lam(Scope {
                unsafe_pattern: scope.unsafe_pattern.clone(),
                unsafe_body: scope.unsafe_body.substs(mappings),
            })),
            Expr::App(ref fun, ref args) => RcExpr::from(Expr::App(
                fun.substs(mappings),
                args.iter().map(|arg| arg.substs(mappings)).collect(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalError {
    ArgumentCountMismatch { expected: usize, given: usize },
}

/// Evaluate an expression into its normal form
pub fn eval(expr: &RcExpr) -> Result<RcExpr, EvalError> {
    match *expr.inner {
        Expr::Var(_) | Expr::Lam(_) => Ok(expr.clone()),
        Expr::App(ref fun, ref args) => match *eval(fun)?.inner {
            Expr::Lam(ref scope) => {
                let (binders, body) = scope.clone().unbind();

                if binders.len() != args.len() {
                    Err(EvalError::ArgumentCountMismatch {
                        expected: binders.len(),
                        given: args.len(),
                    })
                } else {
                    let mappings = <_>::zip(
                        binders.into_iter(),
                        args.iter().map(|arg| eval(arg).unwrap()),
                    ).collect::<Vec<_>>();

                    eval(&body.substs(&mappings))
                }
            },
            _ => Ok(expr.clone()),
        },
    }
}

#[test]
fn test_eval_const_lhs() {
    use moniker::FreeVar;

    let x = FreeVar::fresh_named("x");
    let y = FreeVar::fresh_named("y");
    let a = FreeVar::fresh_named("a");
    let b = FreeVar::fresh_named("b");

    // expr = (\(x, y) => y)(a, b)
    let expr = RcExpr::from(Expr::App(
        RcExpr::from(Expr::Lam(Scope::new(
            vec![Binder(x.clone()), Binder(y.clone())],
            RcExpr::from(Expr::Var(Var::Free(y.clone()))),
        ))),
        vec![
            RcExpr::from(Expr::Var(Var::Free(a.clone()))),
            RcExpr::from(Expr::Var(Var::Free(b.clone()))),
        ],
    ));

    assert_term_eq!(
        eval(&expr).unwrap(),
        RcExpr::from(Expr::Var(Var::Free(b.clone()))),
    );
}

#[test]
fn test_eval_const_rhs() {
    use moniker::FreeVar;

    let x = FreeVar::fresh_named("x");
    let y = FreeVar::fresh_named("y");
    let a = FreeVar::fresh_named("a");
    let b = FreeVar::fresh_named("b");

    // expr = (\(x, y) => x)(a, b)
    let expr = RcExpr::from(Expr::App(
        RcExpr::from(Expr::Lam(Scope::new(
            vec![Binder(x.clone()), Binder(y.clone())],
            RcExpr::from(Expr::Var(Var::Free(x.clone()))),
        ))),
        vec![
            RcExpr::from(Expr::Var(Var::Free(a.clone()))),
            RcExpr::from(Expr::Var(Var::Free(b.clone()))),
        ],
    ));

    assert_term_eq!(
        eval(&expr).unwrap(),
        RcExpr::from(Expr::Var(Var::Free(a.clone()))),
    );
}

fn main() {}
