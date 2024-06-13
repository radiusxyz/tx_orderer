use std::{mem::MaybeUninit, sync::Once};

use crate::{error::Error, Database};

static mut DATABASE: MaybeUninit<Database> = MaybeUninit::uninit();
static INIT: Once = Once::new();

impl crate::Database {
    pub fn init(self) {
        unsafe {
            INIT.call_once(|| {
                DATABASE.write(self);
            });
        }
    }
}

pub fn database() -> Result<&'static Database, Error> {
    match INIT.is_completed() {
        true => unsafe { Ok(DATABASE.assume_init_ref()) },
        false => Err(Error::custom(
            crate::ErrorKind::Initialize,
            "Database has not been initialized. Make sure to call `init()` on a `Database` instance",
        )),
    }
}
