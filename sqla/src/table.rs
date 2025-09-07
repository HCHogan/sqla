pub trait TableMeta {
    type Proxy; // all columns NotNull
    type NullableProxy; // all columns Nullable
    const NAME: &'static str;
    fn proxy() -> Self::Proxy;
    fn nullable_proxy() -> Self::NullableProxy;
}
