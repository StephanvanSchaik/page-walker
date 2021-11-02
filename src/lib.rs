//! This crate implements a generic page table walker in Rust, which can be used to either
//! introspect or manage virtual address spaces on architectures that implement a Memory Management
//! Unit (MMU) that traverses a hierarchy of page tables to translate virtual address into physical
//! addresses and a set of permissions. Note that paging is not limited to CPUs, and that paging is
//! also common on modern GPUs. The implementations provided here may therefore be useful when
//! implementing drivers for any sort of paging architecture, an operating system, a hypervisor,
//! etc.
//!
//! The page table hierarchies of different architectures are described in the [`arch`] module. In
//! particular, the [`PageFormat`] struct is used to describe a page table hierarchy or layout
//! consisting of one or more [`PageLevel`] structs, where each level describes which virtual
//! address bits are used to index into the page table. [`PageFormat::walk`] and
//! [`PageFormat::walk_mut`] implement a software page table walker that essentially starts at the
//! root address and traverses the page tables one by one using the [`PageFormat`] struct to select
//! the appropriate bits from the virtual address to index into these page tables.
//!
//! While traversing the page tables, the [`PageFormat::walk`] and [`PageFormat::walk_mut`] invoke
//! functions provided by a user supplied type implementing the [`PageWalker`] and
//! [`PageWalkerMut`] traits respectively to operate on the various page table entries (PTEs). Note
//! that there is an immutable version that does not allow modification of the page tables, and a
//! mutable version that does.
//!
//! While it is possible to implement your own [`PageWalker`] and [`PageWalkerMut`], this crate
//! also provides a higher-level abstraction of an virtual address space in [`AddressSpace`] that
//! only requires you to implement a [`PageTableMapper`] for mapping and unmapping page tables. The
//! [`AddressSpace`] then simply offers you the functionality to retrieve and modify the PTEs of
//! existing pages.

#![no_std]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

pub mod address_space;
pub mod arch;
pub mod format;
pub mod level;
pub mod table;
pub mod walker;
pub mod walkers;

pub use address_space::{AddressSpace, PageTableMapper};
pub use format::PageFormat;
pub use level::PageLevel;
pub use table::{PageTable, PageTableMut};
pub use walker::{PageWalker, PageWalkerMut, PteType};
