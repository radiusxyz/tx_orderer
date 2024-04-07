#[macro_export]
macro_rules! check_trait_bound {
    ($ty:ty, $tr:ident) => {
        || (
            fn check_trait_bound<T: $tr>() {}
            check_trait_bound::<$ty>()
        )
    };
}
