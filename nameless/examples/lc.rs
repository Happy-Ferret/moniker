//! An example of using the `nameless` library to implement the untyped lambda
//! calculus

#[macro_use]
extern crate nameless;

use nameless::{FreeVar, Scope, Var};
use std::rc::Rc;

#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    Var(Var),
    Lam(Scope<FreeVar, Rc<Expr>>),
    App(Rc<Expr>, Rc<Expr>),
}

// FIXME: auto-derive this somehow!
fn subst(expr: &Rc<Expr>, subst_name: &FreeVar, subst_expr: &Rc<Expr>) -> Rc<Expr> {
    match **expr {
        Expr::Var(Var::Free(ref n)) if subst_name == n => subst_expr.clone(),
        Expr::Var(_) => expr.clone(),
        Expr::Lam(ref scope) => Rc::new(Expr::Lam(Scope {
            unsafe_pattern: scope.unsafe_pattern.clone(),
            unsafe_body: subst(&scope.unsafe_body, subst_name, subst_expr),
        })),
        Expr::App(ref fun, ref arg) => Rc::new(Expr::App(
            subst(fun, subst_name, subst_expr),
            subst(arg, subst_name, subst_expr),
        )),
    }
}

pub fn eval(expr: &Rc<Expr>) -> Rc<Expr> {
    match **expr {
        Expr::Var(Var::Free(_)) => expr.clone(),
        Expr::Var(Var::Bound(ref name, _)) => panic!("encountered a bound variable: {:?}", name),
        Expr::Lam(_) => expr.clone(),
        Expr::App(ref fun, ref arg) => match *eval(fun) {
            Expr::Lam(ref scope) => {
                let (name, body) = scope.clone().unbind();
                eval(&subst(&body, &name, &eval(arg)))
            },
            _ => expr.clone(),
        },
    }
}

#[test]
fn test_eval() {
    // expr = (\x -> x) y
    let expr = Rc::new(Expr::App(
        Rc::new(Expr::Lam(Scope::new(
            FreeVar::user("x"),
            Rc::new(Expr::Var(Var::Free(FreeVar::user("x")))),
        ))),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("y")))),
    ));

    assert_term_eq!(
        eval(&expr),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("y")))),
    );
}

fn main() {}
