//! An example of using the `moniker` library to implement the simply typed
//! lambda calculus with records, variants, and iso-recursive types.
//!
//! We use [bidirectional type checking](http://www.davidchristiansen.dk/tutorials/bidirectional.pdf)
//! to get some level of type inference.

extern crate im;
#[macro_use]
extern crate moniker;

use im::HashMap;
use moniker::{Binder, BoundTerm, Embed, FreeVar, Rec, Scope, Var};
use std::rc::Rc;

/// Types
#[derive(Debug, Clone, BoundTerm)]
pub enum Type {
    /// Integers
    Int,
    /// Floating point numbers
    Float,
    /// Strings
    String,
    /// Type variables
    Var(Var<String>), // TODO: Separate identifier namespaces? See issue #8
    /// Function types
    Arrow(RcType, RcType),
    /// Record types
    Record(Vec<(String, RcType)>),
    /// Variant types
    Variant(Vec<(String, RcType)>),
    /// Recursive types
    Rec(Scope<Rec<(Binder<String>, Embed<RcType>)>, ()>),
}

/// Reference counted types
#[derive(Debug, Clone, BoundTerm)]
pub struct RcType {
    pub inner: Rc<Type>,
}

impl From<Type> for RcType {
    fn from(src: Type) -> RcType {
        RcType {
            inner: Rc::new(src),
        }
    }
}

impl RcType {
    // FIXME: auto-derive this somehow!
    fn subst<N>(&self, name: &N, replacement: &RcType) -> RcType
    where
        Var<String>: PartialEq<N>,
    {
        match *self.inner {
            Type::Var(ref var) if var == name => replacement.clone(),
            Type::Var(_) | Type::Int | Type::Float | Type::String => self.clone(),
            Type::Arrow(ref param, ref body) => RcType::from(Type::Arrow(
                param.subst(name, replacement),
                body.subst(name, replacement),
            )),
            Type::Record(ref fields) => {
                let fields = fields
                    .iter()
                    .map(|&(ref label, ref elem)| (label.clone(), elem.subst(name, replacement)))
                    .collect();

                RcType::from(Type::Record(fields))
            },
            Type::Variant(ref variants) => {
                let variants = variants
                    .iter()
                    .map(|&(ref label, ref elem)| (label.clone(), elem.subst(name, replacement)))
                    .collect();

                RcType::from(Type::Variant(variants))
            },
            Type::Rec(ref scope) => {
                let (ref n, Embed(ref ann)) = scope.unsafe_pattern.unsafe_pattern;
                RcType::from(Type::Rec(Scope {
                    unsafe_pattern: Rec {
                        unsafe_pattern: (n.clone(), Embed(ann.subst(name, replacement))),
                    },
                    unsafe_body: (),
                }))
            },
        }
    }
}

/// Literal values
#[derive(Debug, Clone, BoundTerm)]
pub enum Literal {
    /// Integer literals
    Int(i32),
    /// Floating point literals
    Float(f32),
    /// String literals
    String(String),
}

/// Expressions
#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    /// Annotated expressions
    Ann(RcExpr, RcType),
    /// Literals
    Literal(Literal),
    /// Variables
    Var(Var<String>), // TODO: Separate identifier namespaces? See issue #8
    /// Lambda expressions, with an optional type annotation for the parameter
    Lam(Scope<(Binder<String>, Embed<Option<RcType>>), RcExpr>),
    /// Function application
    App(RcExpr, RcExpr),
    /// Record values
    Record(Vec<(String, RcExpr)>),
    /// Field projection on records
    Proj(RcExpr, String),
    /// Variant introduction
    Tag(String, RcExpr),
    /// Fold a recursive type
    Fold(RcType, RcExpr),
    /// Unfold a recursive type
    Unfold(RcType, RcExpr),
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
            Expr::Ann(ref expr, ref ty) => {
                RcExpr::from(Expr::Ann(expr.subst(name, replacement), ty.clone()))
            },
            Expr::Var(ref var) if var == name => replacement.clone(),
            Expr::Var(_) | Expr::Literal(_) => self.clone(),
            Expr::Lam(ref scope) => RcExpr::from(Expr::Lam(Scope {
                unsafe_pattern: scope.unsafe_pattern.clone(),
                unsafe_body: scope.unsafe_body.subst(name, replacement),
            })),
            Expr::App(ref fun, ref arg) => RcExpr::from(Expr::App(
                fun.subst(name, replacement),
                arg.subst(name, replacement),
            )),
            Expr::Record(ref fields) => {
                let fields = fields
                    .iter()
                    .map(|&(ref label, ref elem)| (label.clone(), elem.subst(name, replacement)))
                    .collect();

                RcExpr::from(Expr::Record(fields))
            },
            Expr::Proj(ref expr, ref label) => {
                RcExpr::from(Expr::Proj(expr.subst(name, replacement), label.clone()))
            },
            Expr::Tag(ref label, ref expr) => {
                RcExpr::from(Expr::Tag(label.clone(), expr.subst(name, replacement)))
            },
            Expr::Fold(ref ty, ref expr) => {
                RcExpr::from(Expr::Fold(ty.clone(), expr.subst(name, replacement)))
            },
            Expr::Unfold(ref ty, ref expr) => {
                RcExpr::from(Expr::Unfold(ty.clone(), expr.subst(name, replacement)))
            },
        }
    }
}

/// A context containing a series of type annotations
type Context = HashMap<FreeVar<String>, RcType>;

/// Evaluate an expression into its normal form
pub fn eval(expr: &RcExpr) -> RcExpr {
    match *expr.inner {
        Expr::Ann(ref expr, _) => eval(expr),
        Expr::Literal(_) | Expr::Var(_) | Expr::Lam(_) => expr.clone(),
        Expr::App(ref fun, ref arg) => match *eval(fun).inner {
            Expr::Lam(ref scope) => {
                let ((binder, _), body) = scope.clone().unbind();
                eval(&body.subst(&binder, &eval(arg)))
            },
            _ => expr.clone(),
        },
        Expr::Record(ref fields) => {
            let fields = fields
                .iter()
                .map(|&(ref label, ref elem)| (label.clone(), eval(elem)))
                .collect();

            RcExpr::from(Expr::Record(fields))
        },
        Expr::Proj(ref expr, ref label) => {
            let expr = eval(expr);

            if let Expr::Record(ref fields) = *expr.inner {
                if let Some(&(_, ref e)) = fields.iter().find(|&(ref l, _)| l == label) {
                    return e.clone();
                }
            }

            expr
        },
        Expr::Tag(ref label, ref expr) => RcExpr::from(Expr::Tag(label.clone(), eval(expr))),
        Expr::Fold(ref ty, ref expr) => RcExpr::from(Expr::Fold(ty.clone(), eval(expr))),
        Expr::Unfold(ref ty, ref expr) => {
            let expr = eval(expr);
            if let Expr::Fold(_, ref expr) = *expr.inner {
                return expr.clone();
            }
            RcExpr::from(Expr::Unfold(ty.clone(), expr))
        },
    }
}

/// Check that a (potentially ambiguous) expression can conforms to a given
/// expected type
pub fn check(context: &Context, expr: &RcExpr, expected_ty: &RcType) -> Result<(), String> {
    match (&*expr.inner, &*expected_ty.inner) {
        (&Expr::Lam(ref scope), &Type::Arrow(ref param_ty, ref ret_ty)) => {
            if let ((binder, Embed(None)), body) = scope.clone().unbind() {
                // FIXME: Ick!
                let free_var = binder
                    .try_into_free_var()
                    .expect("encountered a bound variable");
                check(&context.insert(free_var, param_ty.clone()), &body, ret_ty)?;
                return Ok(());
            }
        },
        (&Expr::Tag(ref label, ref expr), &Type::Variant(ref variants)) => {
            return match variants.iter().find(|&(l, _)| l == label) {
                None => Err(format!(
                    "variant type did not contain the label `{}`",
                    label
                )),
                Some(&(_, ref ty)) => check(context, expr, ty),
            };
        },
        (_, _) => {},
    }

    let inferred_ty = infer(&context, expr)?;

    if RcType::term_eq(&inferred_ty, expected_ty) {
        Ok(())
    } else {
        Err(format!(
            "type mismatch - found `{:?}` but expected `{:?}`",
            inferred_ty, expected_ty
        ))
    }
}

/// Synthesize the types of unambiguous expressions
pub fn infer(context: &Context, expr: &RcExpr) -> Result<RcType, String> {
    match *expr.inner {
        Expr::Ann(ref expr, ref ty) => {
            check(context, expr, ty)?;
            Ok(ty.clone())
        },
        Expr::Literal(Literal::Int(_)) => Ok(RcType::from(Type::Int)),
        Expr::Literal(Literal::Float(_)) => Ok(RcType::from(Type::Float)),
        Expr::Literal(Literal::String(_)) => Ok(RcType::from(Type::String)),
        Expr::Var(ref var) => match context.get(
            // FIXME: Ick!
            &var.clone()
                .try_into_free_var()
                .expect("encountered a bound variable"),
        ) {
            Some(term) => Ok((*term).clone()),
            None => Err(format!("`{:?}` not found in `{:?}`", var, context)),
        },
        Expr::Lam(ref scope) => match scope.clone().unbind() {
            ((binder, Embed(Some(ann))), body) => {
                // FIXME: Ick!
                let free_var = binder
                    .try_into_free_var()
                    .expect("encountered a bound variable");
                let body_ty = infer(&context.insert(free_var, ann.clone()), &body)?;
                Ok(RcType::from(Type::Arrow(ann, body_ty)))
            },
            ((binder, Embed(None)), _) => Err(format!(
                "type annotation needed for parameter `{:?}`",
                binder
            )),
        },
        Expr::App(ref fun, ref arg) => match *infer(context, fun)?.inner {
            Type::Arrow(ref param_ty, ref ret_ty) => {
                let arg_ty = infer(context, arg)?;
                if RcType::term_eq(param_ty, &arg_ty) {
                    Ok(ret_ty.clone())
                } else {
                    Err(format!(
                        "argument type mismatch - found `{:?}` but expected `{:?}`",
                        arg_ty, param_ty,
                    ))
                }
            },
            _ => Err(format!("`{:?}` is not a function", fun)),
        },
        Expr::Record(ref fields) => Ok(RcType::from(Type::Record(
            fields
                .iter()
                .map(|&(ref label, ref expr)| Ok((label.clone(), infer(context, expr)?)))
                .collect::<Result<_, String>>()?,
        ))),
        Expr::Proj(ref expr, ref label) => match *infer(context, expr)?.inner {
            Type::Record(ref fields) => match fields.iter().find(|&(l, _)| l == label) {
                Some(&(_, ref ty)) => Ok(ty.clone()),
                None => Err(format!("field `{}` not found in type", label)),
            },
            _ => Err("record expected".to_string()),
        },
        Expr::Tag(_, _) => Err("type annotations needed".to_string()),
        Expr::Fold(ref ty, ref expr) => match *ty.inner {
            Type::Rec(ref scope) => {
                let (binder, Embed(body_ty)) = scope.clone().unbind().0.unrec();
                check(context, expr, &body_ty.subst(&binder, ty))?;
                Ok(ty.clone())
            },
            _ => Err(format!("found `{:?}` but expected a recursive type", ty)),
        },
        Expr::Unfold(ref ty, ref expr) => match *ty.inner {
            Type::Rec(ref scope) => {
                let (binder, Embed(body_ty)) = scope.clone().unbind().0.unrec();
                check(context, expr, ty)?;
                Ok(body_ty.subst(&binder, ty))
            },
            _ => Err(format!("found `{:?}` but expected a recursive type", ty)),
        },
    }
}

#[test]
fn test_infer() {
    // expr = (\x : Int -> x)
    let expr = RcExpr::from(Expr::Lam(Scope::new(
        (Binder::user("x"), Embed(Some(RcType::from(Type::Int)))),
        RcExpr::from(Expr::Var(Var::user("x"))),
    )));

    assert_term_eq!(
        infer(&Context::new(), &expr).unwrap(),
        RcType::from(Type::Arrow(
            RcType::from(Type::Int),
            RcType::from(Type::Int)
        )),
    );
}

// TODO: Use property testing for this!
// http://janmidtgaard.dk/papers/Midtgaard-al%3AICFP17-full.pdf

fn main() {}
