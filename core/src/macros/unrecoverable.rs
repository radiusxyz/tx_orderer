#[macro_export]
macro_rules! unrecoverable {
    ($error:expr) => {{
        println!("[Panic]: {:?}", $error);
        std::process::exit(1)
    }};
}
