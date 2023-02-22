#[macro_use]
extern crate alloc;

pub mod without_std {
    pub use alloc::{borrow, string, sync, vec};
    pub use core::{fmt, mem, num};

    pub mod collections {
        pub use hashbrown::HashMap;
    }
}
