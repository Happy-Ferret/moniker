//! An example of using the `moniker` library to implement the untyped lambda
//! calculus with `letrec` bindings.

#[macro_use]
extern crate moniker;

use moniker::{Binder, BoundTerm, Embed, FreeVar, Rec, Scope, Subst, Var};
use std::rc::Rc;

/// Expressions
#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    /// Variables
    Var(Var<String>),
    /// Lambda expressions
    Lam(Scope<Binder<String>, RcExpr>),
    /// Function application
    App(RcExpr, RcExpr),
    /// Mutually recursive let bindings
    LetRec(Scope<Rec<Vec<(Binder<String>, Embed<RcExpr>)>>, RcExpr>),
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
    fn subst<N>(&self, name: &N, replacement: &RcExpr) -> RcExpr
    where
        Var<String>: PartialEq<N>,
    {
        match *self.inner {
            Expr::Var(ref n) if n == name => replacement.clone(),
            Expr::Var(_) => self.clone(),
            Expr::Lam(ref scope) => RcExpr::from(Expr::Lam(Scope {
                unsafe_pattern: scope.unsafe_pattern.clone(),
                unsafe_body: scope.unsafe_body.subst(name, replacement),
            })),
            Expr::App(ref fun, ref arg) => RcExpr::from(Expr::App(
                fun.subst(name, replacement),
                arg.subst(name, replacement),
            )),
            Expr::LetRec(ref scope) => RcExpr::from(Expr::LetRec(Scope {
                unsafe_pattern: Rec {
                    unsafe_pattern: scope
                        .unsafe_pattern
                        .unsafe_pattern
                        .iter()
                        .map(|&(ref n, Embed(ref value))| {
                            (n.clone(), Embed(value.subst(name, replacement)))
                        })
                        .collect(),
                },
                unsafe_body: scope.unsafe_body.subst(name, replacement),
            })),
        }
    }
}

// TODO: Implement this, then figure out how to derive it!
impl Subst<String, RcExpr> for RcExpr {
    fn subst(&mut self, name: &FreeVar<String>, replacement: &RcExpr) {
        unimplemented!()
    }

    fn substs(&mut self, mappings: &[(FreeVar<String>, RcExpr)]) {
        unimplemented!()
    }
}

/// Evaluate an expression into its normal form
pub fn eval(expr: &RcExpr) -> RcExpr {
    match *expr.inner {
        Expr::Var(_) | Expr::Lam(_) => expr.clone(),
        Expr::App(ref fun, ref arg) => match *eval(fun).inner {
            Expr::Lam(ref scope) => {
                let (name, body) = scope.clone().unbind();
                eval(&body.subst(&name, &eval(arg)))
            },
            _ => expr.clone(),
        },
        Expr::LetRec(ref scope) => {
            let (bindings, mut body) = scope.clone().unbind();
            let bindings = bindings.unrec();

            // substitute the variable definitions all (once) throughout the body
            for &(ref name, Embed(ref binding)) in &bindings {
                body = body.subst(name, binding);
            }

            // garbage collect, if possible
            // FIXME: `free_vars` is slow! We probably want this to be faster - see issue #10
            let fvs = body.free_vars();
            if bindings.iter().any(|&(ref name, _)| match *name {
                Binder::Free(ref name) => fvs.contains(name),
                _ => panic!("encountered a bound variable"),
            }) {
                RcExpr::from(Expr::LetRec(Scope::new(Rec::new(&bindings), body)))
            } else {
                eval(&body)
            }
        },
    }
}

#[test]
fn test_eval() {
    // expr = (\x -> x) y
    let expr = RcExpr::from(Expr::App(
        RcExpr::from(Expr::Lam(Scope::new(
            Binder::user("x"),
            RcExpr::from(Expr::Var(Var::user("x"))),
        ))),
        RcExpr::from(Expr::Var(Var::user("y"))),
    ));

    assert_term_eq!(eval(&expr), RcExpr::from(Expr::Var(Var::user("y"))),);
}

#[test]
fn test_eval_let_rec() {
    // expr =
    //      letrec
    //          test = id x
    //          id =  \x -> x
    //      in
    //          test
    let expr = RcExpr::from(Expr::LetRec(Scope::new(
        Rec::new(&vec![
            (
                Binder::user("test"),
                Embed(RcExpr::from(Expr::App(
                    RcExpr::from(Expr::Var(Var::user("id"))),
                    RcExpr::from(Expr::Var(Var::user("x"))),
                ))),
            ),
            (
                Binder::user("id"),
                Embed(RcExpr::from(Expr::Lam(Scope::new(
                    Binder::user("x"),
                    RcExpr::from(Expr::Var(Var::user("x"))),
                )))),
            ),
        ]),
        RcExpr::from(Expr::Var(Var::user("test"))),
    )));

    assert_term_eq!(eval(&expr), RcExpr::from(Expr::Var(Var::user("x"))));
}

fn main() {}
