//! This module provides various walkers to help with managing the [`AddressSpace`].
//!
//! [`AddressSpace`]: `super::AddressSpace`

pub mod allocator;
pub mod mapper;
pub mod protector;
pub mod reader;
pub mod remover;
pub mod writer;

pub use allocator::PteAllocator;
pub use mapper::PteMapper;
pub use protector::PteProtector;
pub use reader::PteReader;
pub use remover::{PteRemovalFlags, PteRemover};
pub use writer::PteWriter;
