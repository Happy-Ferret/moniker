//! An example of using the `nameless` library to implement the untyped lambda
//! calculus with multibinders

#[macro_use]
extern crate nameless;

use std::rc::Rc;
use nameless::{BoundTerm, Name, Scope, Var};

#[derive(Debug, Clone)]
pub enum Env {
    Empty,
    Extend(Rc<Env>, Name, Rc<Expr>),
}

fn extend(env: Rc<Env>, name: Name, expr: Rc<Expr>) -> Rc<Env> {
    Rc::new(Env::Extend(env, name, expr))
}

fn lookup<'a>(mut env: &'a Rc<Env>, name: &Name) -> Option<&'a Rc<Expr>> {
    while let Env::Extend(ref next_env, ref curr_name, ref expr) = **env {
        if Name::term_eq(curr_name, name) {
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
    Lam(Scope<Vec<Name>, Rc<Expr>>),
    App(Rc<Expr>, Vec<Rc<Expr>>),
}

#[derive(Debug, Clone)]
pub enum EvalError {
    ArgumentCountMismatch { expected: usize, given: usize },
}

pub fn eval(env: &Rc<Env>, expr: &Rc<Expr>) -> Result<Rc<Expr>, EvalError> {
    match **expr {
        Expr::Var(Var::Free(ref name)) => Ok(lookup(env, name).unwrap_or(expr).clone()),
        Expr::Var(Var::Bound(ref name, _)) => panic!("encountered a bound variable: {:?}", name),
        Expr::Lam(_) => Ok(expr.clone()),
        Expr::App(ref fun, ref args) => match *eval(env, fun)? {
            Expr::Lam(ref scope) => {
                let (params, body) = nameless::unbind(scope.clone());

                if params.len() != args.len() {
                    Err(EvalError::ArgumentCountMismatch {
                        expected: params.len(),
                        given: args.len(),
                    })
                } else {
                    let mut acc_env = env.clone();
                    for (param_name, arg) in <_>::zip(params.into_iter(), args.iter()) {
                        acc_env = extend(acc_env, param_name, eval(env, arg)?);
                    }
                    eval(&acc_env, &body)
                }
            },
            _ => Ok(expr.clone()),
        },
    }
}

#[test]
fn test_eval() {
    // expr = (fn(x, y) -> y)(a, b)
    let expr = Rc::new(Expr::App(
        Rc::new(Expr::Lam(Scope::bind(
            vec![Name::user("x"), Name::user("y")],
            Rc::new(Expr::Var(Var::Free(Name::user("y")))),
        ))),
        vec![
            Rc::new(Expr::Var(Var::Free(Name::user("a")))),
            Rc::new(Expr::Var(Var::Free(Name::user("b")))),
        ],
    ));

    assert_term_eq!(
        eval(&Rc::new(Env::Empty), &expr).unwrap(),
        Rc::new(Expr::Var(Var::Free(Name::user("b")))),
    );
}

fn main() {}
