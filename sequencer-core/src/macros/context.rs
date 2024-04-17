#[macro_export]
macro_rules! context {
    ($($argument:expr),* $(,)?) => {{
        use std::fmt::Write;

        let mut context = String::default();
        $(
            writeln!(&mut context, "\t{}: {:?}", stringify!($argument), $argument).unwrap();
        )*
        context
    }};
}
