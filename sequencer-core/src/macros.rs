#[macro_export]
macro_rules! error_context {
    ($($argument:expr),* $(,)?) => {{
        let mut context = String::new();
        $(
            write!(&mut context as &mut dyn std::fmt::Write, "\n\t{}: {:?}", stringify!($argument), $argument).unwrap();
        )*
        context
    }};
}
