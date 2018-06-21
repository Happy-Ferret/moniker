//! An example of using the `nameless` library to implement the untyped lambda
//! calculus

#[macro_use]
extern crate nameless;

use nameless::{Bind, BoundTerm, Embed, FreeVar, Nest, Var};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Env {
    Empty,
    Extend(Rc<Env>, FreeVar, Rc<Expr>),
}

fn extend(env: Rc<Env>, name: FreeVar, expr: Rc<Expr>) -> Rc<Env> {
    Rc::new(Env::Extend(env, name, expr))
}

fn lookup<'a>(mut env: &'a Rc<Env>, name: &FreeVar) -> Option<&'a Rc<Expr>> {
    while let Env::Extend(ref next_env, ref curr_name, ref expr) = **env {
        if FreeVar::term_eq(curr_name, name) {
            return Some(expr);
        } else {
            env = next_env;
        }
    }
    None
}

#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    Var(Var),
    Lam(Bind<FreeVar, Rc<Expr>>),
    Let(Bind<Nest<(FreeVar, Embed<Rc<Expr>>)>, Rc<Expr>>),
    App(Rc<Expr>, Rc<Expr>),
}

pub fn eval(env: &Rc<Env>, expr: &Rc<Expr>) -> Rc<Expr> {
    match **expr {
        Expr::Var(Var::Free(ref name)) => lookup(env, name).unwrap_or(expr).clone(),
        Expr::Var(Var::Bound(ref name, _)) => panic!("encountered a bound variable: {:?}", name),
        Expr::Lam(_) => expr.clone(),
        Expr::Let(ref scope) => {
            let (bindings, body) = scope.clone().unbind();
            let mut env = env.clone();
            for (name, Embed(value)) in bindings.unnest() {
                let value = eval(&env, &value);
                env = extend(env, name, value);
            }
            eval(&env, &body)
        },
        Expr::App(ref fun, ref arg) => match *eval(env, fun) {
            Expr::Lam(ref scope) => {
                let (name, body) = scope.clone().unbind();
                eval(&extend(env.clone(), name, eval(env, arg)), &body)
            },
            _ => expr.clone(),
        },
    }
}

#[test]
fn test_eval() {
    // expr = (\x -> x) y
    let expr = Rc::new(Expr::App(
        Rc::new(Expr::Lam(Bind::new(
            FreeVar::user("x"),
            Rc::new(Expr::Var(Var::Free(FreeVar::user("x")))),
        ))),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("y")))),
    ));

    assert_term_eq!(
        eval(&Rc::new(Env::Empty), &expr),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("y")))),
    );
}

#[test]
fn test_eval_let() {
    // expr =
    //      let id = \x -> x
    //          foo =  y
    //          bar = id foo
    //      in bar
    let expr = Rc::new(Expr::Let(Bind::new(
        Nest::new(vec![
            (
                FreeVar::user("id"),
                Embed(Rc::new(Expr::Lam(Bind::new(
                    FreeVar::user("x"),
                    Rc::new(Expr::Var(Var::Free(FreeVar::user("x")))),
                )))),
            ),
            (
                FreeVar::user("foo"),
                Embed(Rc::new(Expr::Var(Var::Free(FreeVar::user("y"))))),
            ),
            (
                FreeVar::user("bar"),
                Embed(Rc::new(Expr::App(
                    Rc::new(Expr::Var(Var::Free(FreeVar::user("id")))),
                    Rc::new(Expr::Var(Var::Free(FreeVar::user("foo")))),
                ))),
            ),
        ]),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("bar")))),
    )));

    assert_term_eq!(
        eval(&Rc::new(Env::Empty), &expr),
        Rc::new(Expr::Var(Var::Free(FreeVar::user("y")))),
    );
}

fn main() {}
