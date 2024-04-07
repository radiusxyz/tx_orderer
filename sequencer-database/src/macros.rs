#[macro_export]
macro_rules! key_list {
    [$(&$key:expr),+ $(,)?] => {{
        (|| {
            let mut key_list: Vec<Vec<u8>> = Vec::default();

            $(
                let key_vec = sequencer_framework::bincode::serialize(&$key).map_err(|error| {
                    sequencer_framework::error::Error::new_with_context(
                        sequencer_framework::caller!(key_list!()),
                        format_args!("key: {:?}", &$key),
                        error,
                    )
                })?;
                key_list.push(key_vec);
            )+

            Ok(key_list)
        })()
    }};
}

#[macro_export]
macro_rules! key_value_list {
    [$(($key:expr, $value:expr)),+ $(,)?] => {{
        (|| {
            let mut key_value_list: Vec<(Vec<u8>, Vec<u8>)> = Vec::default();

            $(
                let key_vec = sequencer_framework::bincode::serialize(&$key).map_err(|error| {
                    sequencer_framework::error::Error::new_with_context(
                        sequencer_framework::caller!(key_value_list!()),
                        format_args!("key: {:?}", &$value),
                        error,
                    )
                })?;
                let value_vec = sequencer_framework::bincode::serialize(&$value).map_err(|error| {
                    sequencer_framework::error::Error::new_with_context(
                        sequencer_framework::caller!(key_value_list!()),
                        format_args!("value: {:?}", &$value),
                        error,
                    )
                })?;
                key_value_list.push((key_vec, value_vec));
            )+

            Ok(key_value_list)
        })()
    }};
}
