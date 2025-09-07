use crate::ast::AstFrom;
use crate::table::TableMeta;
use core::marker::PhantomData;

pub struct Join<L, R>(pub PhantomData<(L, R)>);

pub struct JoinProxy<Lp, Rp> {
    pub l: Lp,
    pub r: Rp,
}

pub trait HasFromAst {
    fn from_ast() -> AstFrom;
}

impl<T> HasFromAst for T
where
    T: TableMeta,
{
    fn from_ast() -> AstFrom {
        AstFrom::Table { name: T::NAME }
    }
}

impl<L, R> HasFromAst for Join<L, R>
where
    L: TableMeta,
    R: TableMeta,
{
    fn from_ast() -> AstFrom {
        // a bare Join has no ON; the query builder will add it when left_join is called
        AstFrom::Table { name: L::NAME } // placeholder, actual build happens in left_join
    }
}

pub trait HasProxy {
    type Proxy;
    fn proxy() -> Self::Proxy;
}

impl<T> HasProxy for T
where
    T: TableMeta,
{
    type Proxy = <T as TableMeta>::Proxy;
    fn proxy() -> Self::Proxy {
        <T as TableMeta>::proxy()
    }
}

impl<L, R> HasProxy for Join<L, R>
where
    L: TableMeta,
    R: TableMeta,
{
    type Proxy = JoinProxy<<L as TableMeta>::Proxy, <R as TableMeta>::NullableProxy>;
    fn proxy() -> Self::Proxy {
        JoinProxy {
            l: L::proxy(),
            r: R::nullable_proxy(),
        }
    }
}
