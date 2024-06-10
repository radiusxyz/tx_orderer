use std::{mem::MaybeUninit, sync::Once};

use crate::Database;

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

pub fn database() -> &'static Database {
    if INIT.is_completed() {
        unsafe { DATABASE.assume_init_ref() }
    } else {
        panic!("Database has not been initialized.");
    }
}
