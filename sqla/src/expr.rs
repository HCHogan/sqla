#![allow(non_camel_case_types)]
use crate::ast::AstExpr;
use crate::col::Col;
use crate::col::ColumnMeta;
use crate::table::TableMeta;
use crate::types::{NonAgg, NotNull, Nullable};
use core::marker::PhantomData;
use std::ops::{BitAnd, BitOr, Not};

pub struct Expr<SqlTy, Null, AggState> {
    pub(crate) ast: AstExpr,
    pub(crate) _p: PhantomData<(SqlTy, Null, AggState)>,
}

impl<SqlTy, Null, AggState> Clone for Expr<SqlTy, Null, AggState> {
    fn clone(&self) -> Self {
        Self {
            ast: self.ast.clone(),
            _p: PhantomData,
        }
    }
}

pub trait AsExpr {
    type Ty;
    type Null; // NotNull | Nullable
    type Agg; // NonAgg | Agg
    fn to_expr(self) -> Expr<Self::Ty, Self::Null, Self::Agg>;
}

impl<T, N, A> AsExpr for Expr<T, N, A> {
    type Ty = T;
    type Null = N;
    type Agg = A;
    fn to_expr(self) -> Expr<T, N, A> {
        self
    }
}
impl<T, N, A> AsExpr for &Expr<T, N, A> {
    type Ty = T;
    type Null = N;
    type Agg = A;
    fn to_expr(self) -> Expr<T, N, A> {
        (*self).clone()
    }
}

impl<Table, Ty, Null, Tag> AsExpr for Col<Table, Ty, Null, Tag>
where
    Table: TableMeta,
    Tag: ColumnMeta,
{
    type Ty = Ty;
    type Null = Null;
    type Agg = NonAgg;
    fn to_expr(self) -> Expr<Ty, Null, NonAgg> {
        self.expr()
    }
}
impl<Table, Ty, Null, Tag> AsExpr for &Col<Table, Ty, Null, Tag>
where
    Table: TableMeta,
    Tag: ColumnMeta,
{
    type Ty = Ty;
    type Null = Null;
    type Agg = NonAgg;
    fn to_expr(self) -> Expr<Ty, Null, NonAgg> {
        self.expr()
    }
}

impl AsExpr for bool {
    type Ty = bool;
    type Null = NotNull;
    type Agg = NonAgg;
    fn to_expr(self) -> Expr<bool, NotNull, NonAgg> {
        Expr {
            ast: AstExpr::Bool(self),
            _p: PhantomData,
        }
    }
}
impl AsExpr for i64 {
    type Ty = i64;
    type Null = NotNull;
    type Agg = NonAgg;
    fn to_expr(self) -> Expr<i64, NotNull, NonAgg> {
        Expr {
            ast: AstExpr::Int(self),
            _p: PhantomData,
        }
    }
}
impl AsExpr for &'static str {
    type Ty = String;
    type Null = NotNull;
    type Agg = NonAgg;
    fn to_expr(self) -> Expr<String, NotNull, NonAgg> {
        Expr {
            ast: AstExpr::Str(self),
            _p: PhantomData,
        }
    }
}

pub fn lit(v: bool) -> Expr<bool, NotNull, NonAgg> {
    Expr {
        ast: AstExpr::Bool(v),
        _p: PhantomData,
    }
}

impl<T, N, A> Expr<T, N, A> {
    pub fn bin(self, op: &'static str, other: Expr<T, N, A>) -> Expr<bool, NotNull, A> {
        Expr {
            ast: AstExpr::BinOp {
                op,
                lhs: Box::new(self.ast),
                rhs: Box::new(other.ast),
            },
            _p: PhantomData,
        }
    }
}

impl<T, NL, A> Expr<T, NL, A> {
    pub fn eq<NR>(self, rhs: Expr<T, NR, A>) -> Expr<bool, NotNull, A> {
        Expr {
            ast: AstExpr::BinOp {
                op: "=",
                lhs: Box::new(self.ast),
                rhs: Box::new(rhs.ast),
            },
            _p: PhantomData,
        }
    }
    pub fn ne<NR>(self, rhs: Expr<T, NR, A>) -> Expr<bool, NotNull, A> {
        Expr {
            ast: AstExpr::BinOp {
                op: "<>",
                lhs: Box::new(self.ast),
                rhs: Box::new(rhs.ast),
            },
            _p: PhantomData,
        }
    }
}

pub trait EqOps<Rhs>: AsExpr
where
    Rhs: AsExpr<Ty = Self::Ty, Agg = Self::Agg>,
{
    fn eq(self, rhs: Rhs) -> Expr<bool, NotNull, Self::Agg>;
    fn ne(self, rhs: Rhs) -> Expr<bool, NotNull, Self::Agg>;
}

impl<L, R> EqOps<R> for L
where
    L: AsExpr,
    R: AsExpr<Ty = L::Ty, Agg = L::Agg>,
{
    fn eq(self, rhs: R) -> Expr<bool, NotNull, L::Agg> {
        let l = self.to_expr();
        let r = rhs.to_expr();
        Expr {
            ast: AstExpr::BinOp {
                op: "=",
                lhs: Box::new(l.ast),
                rhs: Box::new(r.ast),
            },
            _p: PhantomData,
        }
    }
    fn ne(self, rhs: R) -> Expr<bool, NotNull, L::Agg> {
        let l = self.to_expr();
        let r = rhs.to_expr();
        Expr {
            ast: AstExpr::BinOp {
                op: "<>",
                lhs: Box::new(l.ast),
                rhs: Box::new(r.ast),
            },
            _p: PhantomData,
        }
    }
}

pub trait AndNull<NL, NR> {
    type Out;
}
impl AndNull<NotNull, NotNull> for () {
    type Out = NotNull;
}
impl AndNull<Nullable, NotNull> for () {
    type Out = Nullable;
}
impl AndNull<NotNull, Nullable> for () {
    type Out = Nullable;
}
impl AndNull<Nullable, Nullable> for () {
    type Out = Nullable;
}

impl<NL, NR, A> BitAnd<Expr<bool, NR, A>> for Expr<bool, NL, A>
where
    (): AndNull<NL, NR>,
{
    type Output = Expr<bool, <() as AndNull<NL, NR>>::Out, A>;
    fn bitand(self, rhs: Expr<bool, NR, A>) -> Self::Output {
        Expr {
            ast: AstExpr::BinOp {
                op: "AND",
                lhs: Box::new(self.ast),
                rhs: Box::new(rhs.ast),
            },
            _p: PhantomData,
        }
    }
}
impl<NL, NR, A> BitOr<Expr<bool, NR, A>> for Expr<bool, NL, A>
where
    (): AndNull<NL, NR>,
{
    type Output = Expr<bool, <() as AndNull<NL, NR>>::Out, A>;
    fn bitor(self, rhs: Expr<bool, NR, A>) -> Self::Output {
        Expr {
            ast: AstExpr::BinOp {
                op: "OR",
                lhs: Box::new(self.ast),
                rhs: Box::new(rhs.ast),
            },
            _p: PhantomData,
        }
    }
}
impl<N, A> Not for Expr<bool, N, A> {
    type Output = Expr<bool, N, A>;
    fn not(self) -> Self::Output {
        Expr {
            ast: AstExpr::BinOp {
                op: "NOT",
                lhs: Box::new(self.ast),
                rhs: Box::new(AstExpr::Bool(true)),
            },
            _p: PhantomData,
        }
    }
}

// Runtime row type mapping
pub trait ValueOf<Null, Ty> {
    type Out;
}
impl<Ty> ValueOf<NotNull, Ty> for () {
    type Out = Ty;
}
impl<Ty> ValueOf<Nullable, Ty> for () {
    type Out = Option<Ty>;
}

pub trait ValueType {
    type Out;
}
impl<T, N, A> ValueType for Expr<T, N, A>
where
    (): ValueOf<N, T>,
{
    type Out = <() as ValueOf<N, T>>::Out;
}
impl<Table, Ty, Null, Tag> ValueType for Col<Table, Ty, Null, Tag>
where
    (): ValueOf<Null, Ty>,
{
    type Out = <() as ValueOf<Null, Ty>>::Out;
}

pub trait Projection {
    type Row;
    fn into_vec(self) -> Vec<AstExpr>;
}

macro_rules! impl_projection_for_tuple {
    ( $( $name:ident ),+ ) => {
        impl<$( $name ),+> Projection for ( $( $name, )+ )
        where $( $name: ValueType + crate::expr::AsExpr ),+
        {
            type Row = ( $( <$name as ValueType>::Out, )+ );
            fn into_vec(self) -> Vec<AstExpr> {
                let ( $( $name, )+ ) = self;

                let mut v = Vec::with_capacity(<[()]>::len(&[
                    $( { let _ = &$name; }, )*
                ]));

                $( v.push(crate::expr::AsExpr::to_expr($name).ast); )+
                v
            }
        }
    };
}

impl_projection_for_tuple!(a);
impl_projection_for_tuple!(a, b);
impl_projection_for_tuple!(a, b, c);
impl_projection_for_tuple!(a, b, c, d);
