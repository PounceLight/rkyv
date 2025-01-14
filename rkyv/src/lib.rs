//! rkyv is a zero-copy deserialization framework for Rust.
//!
//! ## Overview
//!
//! rkyv uses Rust's powerful trait system to serialize data without reflection.
//! Many zero-copy deserialization frameworks use external schemas and heavily
//! restrict the available data types. By contrast, rkyv allows all serialized
//! types to be defined in code and can serialize a wide variety of types that
//! other frameworks cannot.
//!
//! rkyv scales to highly-capable as well as highly-restricted environments. Not
//! only does rkyv support "no-std" builds for targets without a standard
//! library implementation, it also supports "no-alloc" builds for targets where
//! allocations cannot be made.
//!
//! rkyv supports limited in-place data mutation, and so can access and update
//! data without ever deserializing back to native types. When rkyv's in-place
//! mutation is too limited, rkyv also provides ergonomic and performant
//! deserialization back into native types.
//!
//! rkyv prioritizes performance, and is one of the fastest serialization
//! frameworks available. All of rkyv's features can be individually enabled and
//! disabled, so you only pay for what you use. Additionally, all of rkyv's
//! zero-copy types are designed to have little to no overhead. In most cases,
//! rkyv's types will have exactly the same performance as native types.
//!
//! See the [rkyv book] for guide-level documentation and usage examples.
//!
//! [rkyv book]: https://rkyv.org
//!
//! ## Components
//!
//! rkyv has [a hash map implementation] that is built for zero-copy
//! deserialization, with the same lookup and iteration performance as the
//! standard library hash maps. The hash map implementation is based on
//! [Swiss Tables] and uses a target-independent version of FxHash to ensure
//! that all targets compute the same hashes.
//!
//! It also has [a B-tree implementation] that has the same performance
//! characteristics as the standard library B-tree maps. Its compact
//! representation and localized data storage is best-suited for very large
//! amounts of data.
//!
//! rkyv supports [shared pointers] by default, and is able to serialize and
//! deserialize them without duplicating the underlying data. Shared pointers
//! which point to the same data when serialized will still point to the same
//! data when deserialized.
//!
//! Alongside its [unchecked API], rkyv also provides optional [validation] so
//! you can ensure safety and data integrity at the cost of some overhead.
//! Because checking serialized data can generally be done without allocations,
//! the cost of checking and zero-copy access can be much lower than that of
//! traditional deserialization.
//!
//! rkyv is trait-oriented from top to bottom, and is made to be extended with
//! custom and specialized types. Serialization, deserialization, and
//! validation traits all accept generic context types, making it easy to add
//! new capabilities without degrading ergonomics.
//!
//! [a hash map implementation]: collections::swiss_table::ArchivedHashMap
//! [Swiss Tables]: https://abseil.io/about/design/swisstables
//! [a B-tree implementation]: collections::btree_map::ArchivedBTreeMap
//! [shared pointers]: rc
//! [unchecked API]: access_unchecked
//! [validation]: access
//!
//! ## Features
//!
//! rkyv has several feature flags which can be used to modify its behavior.
//! Some feature flags change rkyv's serialized format, which can cause
//! previously-serialized data to become unreadable.
//!
//! By default, rkyv enables the `std`, `alloc`, and `bytecheck` features.
//!
//! ### Format
//!
//! These features control how rkyv formats its serialized data. Enabling and
//! disabling these features may change rkyv's serialized format, and can cause
//! previously-serialized data to become unreadable.
//!
//! If an endianness feature is not enabled, rkyv will use little-endian byte
//! ordering. If a pointer width feature is not enabled, rkyv will serialize
//! `isize` and `usize` as 32-bit integers.
//!
//! - `little_endian`: Forces data serialization to use little-endian byte
//!   ordering. This optimizes serialized data for little-endian architectures.
//! - `big_endian`: Forces data serialization to use big-endian byte ordering.
//!   This optimizes serialized data for big-endian architectures.
//! - `unaligned`: Forces data serialization to use unaligned primitives. This
//!   removes alignment requirements for accessing data and allows rkyv to work
//!   with unaligned data more easily.
//! - `pointer_width_16`: Serializes `isize` and `usize` as 16-bit integers.
//!   This is intended to be used only for small data sizes and may not handle
//!   large amounts of data.
//! - `pointer_width_32`: Serializes `isize` and `usize` as 32-bit integers.
//!   This is a good choice for most data, and balances the storage overhead
//!   with support for large data sizes.
//! - `pointer_width_64`: Serializes `isize` and `usize` as 64-bit integers.
//!   This is intended to be used only for extremely large data sizes and may
//!   cause unnecessary data bloat for smaller amounts of data.
//!
//! ### Functionality
//!
//! These features enable more built-in functionality and provide more powerful
//! and ergonomic APIs. Enabling and disabling these features does not change
//! rkyv's serialized format.
//!
//! - `alloc`: Enables support for the `alloc` crate.
//! - `std`: Enables standard library support.
//! - `bytecheck`: Enables data validation through `bytecheck`.
//!
//! ### Crates
//!
//! rkyv provides integrations for some common crates by default. In the future,
//! crates should depend on rkyv and provide their own integration. Enabling and
//! disabling these features does not change rkyv's serialized format.
//!
//! - [`arrayvec`](https://docs.rs/arrayvec)
//! - [`bytes`](https://docs.rs/bytes)
//! - [`hashbrown`](https://docs.rs/hashbrown)
//! - [`indexmap`](https://docs.rs/indexmap)
//! - [`smallvec`](https://docs.rs/smallvec)
//! - [`smol_str`](https://docs.rs/smol_str)
//! - [`tinyvec`](https://docs.rs/tinyvec)
//! - [`triomphe`](https://docs.rs/triomphe)
//! - [`uuid`](https://docs.rs/uuid)

// Crate attributes

#![deny(
    rustdoc::broken_intra_doc_links,
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unsafe_op_in_unsafe_fn
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(docsrs, not(doctest)), feature(doc_cfg, doc_auto_cfg))]
#![doc(html_favicon_url = r#"
    data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0
    26.458 26.458'%3E%3Cpath d='M0 0v26.458h26.458V0zm9.175 3.772l8.107 8.106
    2.702-2.702 2.702 13.512-13.512-2.702 2.703-2.702-8.107-8.107z'/%3E
    %3C/svg%3E
"#)]
#![doc(html_logo_url = r#"
    data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="100"
    height="100" viewBox="0 0 26.458 26.458"%3E%3Cpath d="M0
    0v26.458h26.458V0zm9.175 3.772l8.107 8.106 2.702-2.702 2.702
    13.512-13.512-2.702 2.703-2.702-8.107-8.107z"/%3E%3C/svg%3E
"#)]
#![cfg_attr(miri, feature(alloc_layout_extra))]

// Extern crates

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
use std as alloc;

// Re-exports
#[cfg(feature = "bytecheck")]
pub use ::bytecheck;
pub use ::munge;
pub use ::ptr_meta;
pub use ::rancor;
pub use ::rend;
pub use ::rkyv_derive::{Archive, Deserialize, Portable, Serialize};

// Modules

mod alias;
#[macro_use]
mod _macros;
pub mod api;
pub mod boxed;
pub mod collections;
pub mod de;
mod fmt;
// This is pretty unfortunate. CStr doesn't rely on the rest of std, but it's
// not in core. If CStr ever gets moved into `core` then this module will no
// longer need cfg(feature = "std")
#[cfg(feature = "std")]
pub mod ffi;
pub mod hash;
mod impls;
pub mod net;
pub mod niche;
pub mod ops;
pub mod option;
pub mod place;
mod polyfill;
pub mod primitive;
pub mod rc;
pub mod rel_ptr;
pub mod result;
pub mod ser;
mod simd;
pub mod string;
pub mod time;
pub mod traits;
pub mod tuple;
pub mod util;
#[cfg(feature = "bytecheck")]
pub mod validation;
pub mod vec;
pub mod with;

// Exports

#[cfg(all(feature = "bytecheck", feature = "alloc"))]
#[doc(inline)]
pub use api::high::{access, access_mut, from_bytes};
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use api::high::{deserialize, from_bytes_unchecked, to_bytes};

#[doc(inline)]
pub use crate::{
    alias::*,
    api::{access_unchecked, access_unchecked_mut},
    place::Place,
    traits::{
        Archive, ArchiveUnsized, Deserialize, DeserializeUnsized, Portable,
        Serialize, SerializeUnsized,
    },
};

// Check endianness feature flag settings

#[cfg(all(feature = "little_endian", feature = "big_endian"))]
core::compiler_error!(
    "\"little_endian\" and \"big_endian\" are mutually-exclusive features. \
     You may need to set `default-features = false` or compile with \
     `--no-default-features`."
);

// Check pointer width feature flag settings

#[cfg(all(
    feature = "pointer_width_16",
    feature = "pointer_width_32",
    not(feature = "pointer_width_64")
))]
core::compile_error!(
    "\"pointer_width_16\" and \"pointer_width_32\" are mutually-exclusive \
     features. You may need to set `default-features = false` or compile with \
     `--no-default-features`."
);
#[cfg(all(
    feature = "pointer_width_16",
    feature = "pointer_width_64",
    not(feature = "pointer_width_32")
))]
core::compile_error!(
    "\"pointer_width_16\" and \"pointer_width_64\" are mutually-exclusive \
     features. You may need to set `default-features = false` or compile with \
     `--no-default-features`."
);
#[cfg(all(
    feature = "pointer_width_32",
    feature = "pointer_width_64",
    not(feature = "pointer_width_16")
))]
core::compile_error!(
    "\"pointer_width_32\" and \"pointer_width_64\" are mutually-exclusive \
     features. You may need to set `default-features = false` or compile with \
     `--no-default-features`."
);
#[cfg(all(
    feature = "pointer_width_16",
    feature = "pointer_width_32",
    feature = "pointer_width_64"
))]
core::compile_error!(
    "\"pointer_width_16\", \"pointer_width_32\", and \"pointer_width_64\" are \
     mutually-exclusive features. You may need to set `default-features = \
     false` or compile with `--no-default-features`."
);
