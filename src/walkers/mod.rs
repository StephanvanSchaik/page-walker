//! This module provides various walkers to help with managing the [`AddressSpace`].
//!
//! [`AddressSpace`]: `super::AddressSpace`

pub mod allocator;
pub mod copy;
pub mod mapper;
pub mod protector;
pub mod reader;
pub mod remover;
pub mod writer;

pub use allocator::PteAllocator;
pub use copy::{CopyFromWalker, CopyToWalker};
pub use mapper::PteMapper;
pub use protector::PteProtector;
pub use reader::PteReader;
pub use remover::{PteRemovalFlags, PteRemover};
pub use writer::PteWriter;
