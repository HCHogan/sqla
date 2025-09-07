use core::marker::PhantomData;

pub struct Active;
pub struct Inactive;

pub struct Conn;

pub struct Txn<State> {
    pub _p: PhantomData<State>,
}

impl Conn {
    pub fn with_txn<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&Txn<Active>) -> T,
    {
        let tx = Txn::<Active> { _p: PhantomData };
        f(&tx)
    }
}
