//! This module provides various walkers to help with managing the [`AddressSpace`].

pub mod allocator;
pub mod mapper;
pub mod protector;
pub mod reader;
pub mod remover;
pub mod writer;

pub(crate) use allocator::PteAllocator;
pub(crate) use mapper::PteMapper;
pub(crate) use protector::PteProtector;
pub(crate) use reader::PteReader;
pub(crate) use remover::{PteRemovalFlags, PteRemover};
pub(crate) use writer::PteWriter;
