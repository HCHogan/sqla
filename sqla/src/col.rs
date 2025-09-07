use crate::{ast::AstExpr, expr::Expr, table::TableMeta, types::NonAgg};
use core::marker::PhantomData;

pub trait ColumnMeta {
    const NAME: &'static str;
}

pub struct Col<Table, SqlTy, Null, ColTag> {
    pub _p: PhantomData<(Table, SqlTy, Null, ColTag)>,
}

impl<Table, SqlTy, Null, ColTag> Copy for Col<Table, SqlTy, Null, ColTag> {}
impl<Table, SqlTy, Null, ColTag> Clone for Col<Table, SqlTy, Null, ColTag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Table, SqlTy, Null, ColTag> Col<Table, SqlTy, Null, ColTag>
where
    Table: TableMeta,
    ColTag: super::col::ColumnMeta,
{
    pub fn expr(&self) -> Expr<SqlTy, Null, NonAgg> {
        Expr {
            ast: AstExpr::Column {
                table: Table::NAME,
                col: ColTag::NAME,
            },
            _p: PhantomData,
        }
    }
}
