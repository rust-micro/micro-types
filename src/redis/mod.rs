//! # Redis Types
//!
//! This crate provides a set of types that can be stored in Redis. The types are:
//!
//! * [bool](redis::Dbool)
//! * Integer types:
//!     * signed Integer: [i8](redis::Di8), [i16](redis::Di16), [i32](redis::Di32), [i64](redis::Di64), [isize](redis::Disize)
//!     * unsigned Integer: [u8](redis::Du8), [u16](redis::Du16), [u32](redis::Du32), [u64](redis::Du64), [usize](redis::Dusize)
//! * [String](redis::DString)
//! * [List](redis::List)
//! * Sync types:
//!     * [Mutex](redis::Mutex)
//!     * [SetLoad](redis::SetLoad)
//!
//! This crate implements the most common traits for the primitive types, so it is frictionless to use them in place.
//! The methods of the types can be seen in the documentation of [Generic](redis::Generic).
//! With this crate it is possible to create multiple services that shares the values via Redis.
//! This is helpful if you want to create a distributed system and run multiple instances of the same service.
//! Or you want to communicate between different services. All this kind of stuff can be done with this crate.
//!
//! # Upcoming Features
//!
//! It will be possible to create happens-before relationships between store and load operations like atomic types.
//!
//! # Usage
//!
//! ```
//! use dtypes::redis::Di32 as i32;
//!
//! let client = redis::Client::open("redis://localhost:6379").unwrap();
//! let mut i32 = i32::with_value(1, "test_add", client.clone());
//!
//! i32 = i32 + i32::with_value(2, "test_add2", client.clone());
//! assert_eq!(i32, 3);
//! ```
//!
//! More examples can be found on the doc pages of the types.
//!
mod bool_type;
mod generic;
mod helper;
mod integer;
mod list;
mod mutex;
mod set_load;
mod string;

pub(crate) use helper::apply_operator;

pub use bool_type::TBool as Dbool;
pub use generic::Generic;
pub use integer::{
    Ti16 as Di16, Ti32 as Di32, Ti64 as Di64, Ti8 as Di8, Tisize as Disize, Tu16 as Du16,
    Tu32 as Du32, Tu64 as Du64, Tu8 as Du8, Tusize as Dusize,
};
pub use list::{List, ListCache, ListIter};
pub use mutex::{Guard, LockError, Mutex};
pub use set_load::{SetLoad, SetLoadError};
pub use string::TString as DString;
