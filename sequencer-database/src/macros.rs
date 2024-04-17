#[macro_export]
macro_rules! key_list {
    [$(&$key:expr),+ $(,)?] => {{
        (|| {
            let mut key_list: Vec<Vec<u8>> = Vec::default();

            $(
                let key_vec = sequencer_framework::bincode::serialize(&$key).map_err(|error| {
                    sequencer_framework::error::Error::boxed(error, Some(sequencer_framework::context!(&$key)))
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
                    sequencer_framework::error::Error::boxed(error, Some(sequencer_framework::context!(&$key)))
                })?;

                let value_vec = sequencer_framework::bincode::serialize(&$value).map_err(|error| {
                    sequencer_framework::error::Error::boxed(error, Some(sequencer_framework::context!(&$value)))
                })?;
                key_value_list.push((key_vec, value_vec));
            )+

            Ok(key_value_list)
        })()
    }};
}
