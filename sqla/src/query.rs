use crate::ast::{AstExpr, AstFrom, AstQuery};
use crate::expr::{Expr, Projection};
use crate::join::{HasProxy, Join};
use crate::table::TableMeta;
use crate::txn::{Active, Txn};
use crate::types::{NonAgg, NotNull};
use core::marker::PhantomData;

#[derive(Clone)]
pub struct FromBuilder<Src> {
    pub(crate) from: AstFrom,
    pub(crate) filter: Option<AstExpr>,
    _p: PhantomData<Src>,
}

pub fn from<T>() -> FromBuilder<T>
where
    T: TableMeta,
{
    FromBuilder {
        from: AstFrom::Table { name: T::NAME },
        filter: None,
        _p: PhantomData,
    }
}

pub trait Filterable: Sized {
    type Src;
    fn with_filter(self, f: AstExpr) -> Self;

    fn filter<F>(self, pred: F) -> Self
    where
        F: for<'a> FnOnce(&'a <Self::Src as HasProxy>::Proxy) -> Expr<bool, NotNull, NonAgg>,
        Self::Src: HasProxy,
    {
        let p = <Self::Src as HasProxy>::proxy();
        let e = pred(&p);
        self.with_filter(e.ast)
    }
}

impl<Src> Filterable for FromBuilder<Src>
where
    Src: HasProxy,
{
    type Src = Src;
    fn with_filter(mut self, f: AstExpr) -> Self {
        self.filter = Some(match self.filter {
            None => f,
            Some(old) => AstExpr::BinOp {
                op: "AND",
                lhs: Box::new(old),
                rhs: Box::new(f),
            },
        });
        self
    }
}

impl<L> FromBuilder<L>
where
    L: TableMeta + HasProxy,
{
    pub fn left_join<R, F>(self, _rhs: FromBuilder<R>, on: F) -> FromBuilder<Join<L, R>>
    where
        R: TableMeta + HasProxy,
        F: for<'a, 'b> FnOnce(
            &'a <L as HasProxy>::Proxy,
            &'b <R as HasProxy>::Proxy,
        ) -> Expr<bool, NotNull, NonAgg>,
    {
        let lp = <L as HasProxy>::proxy();
        let rp = <R as HasProxy>::proxy();
        let on_expr = on(&lp, &rp);
        let new_from = AstFrom::Join {
            kind: "LEFT",
            left: Box::new(self.from),
            right: Box::new(AstFrom::Table { name: R::NAME }),
            on: on_expr.ast,
        };
        FromBuilder {
            from: new_from,
            filter: self.filter,
            _p: PhantomData,
        }
    }
}

pub trait Selectable: Sized {
    type Src;
    fn select<F, Proj>(self, f: F) -> Query<<Proj as Projection>::Row>
    where
        F: for<'a> FnOnce(&'a <Self::Src as HasProxy>::Proxy) -> Proj,
        Proj: Projection,
        Self::Src: HasProxy;
}

impl<Src> Selectable for FromBuilder<Src>
where
    Src: HasProxy,
{
    type Src = Src;
    fn select<F, Proj>(self, f: F) -> Query<<Proj as Projection>::Row>
    where
        F: for<'a> FnOnce(&'a <Src as HasProxy>::Proxy) -> Proj,
        Proj: Projection,
    {
        let p = <Src as HasProxy>::proxy();
        let proj = f(&p);
        let proj_vec = proj.into_vec();
        let ast = AstQuery {
            from: Some(self.from),
            filter: self.filter,
            projection: proj_vec,
        };
        Query {
            ast,
            _p: PhantomData,
        }
    }
}

pub struct Query<RowTy> {
    pub(crate) ast: AstQuery,
    pub(crate) _p: PhantomData<RowTy>,
}

impl<RowTy> Query<RowTy> {
    pub fn sql(&self) -> String {
        self.ast.render_sql()
    }

    pub fn fetch_all(self, _tx: &Txn<Active>) -> Vec<RowTy> {
        vec![]
    }
}
