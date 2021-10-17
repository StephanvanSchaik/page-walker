//! This module provides [`PageTable`] and [`PageTableMut`] traits to represent page tables backed
//! by a mapped physical page, that can then be used by a [`crate::walker::PageWalker`] or
//! [`crate::walker::PageWalkerMut`] to access the PTEs in the page tables.

use num_traits::{PrimInt, Unsigned};

/// This represents a page table backed by a mapped physical page and provides immutable access to
/// the PTEs to [`crate::walker::PageWalker`].
pub trait PageTable<PTE>: AsRef<[PTE]>
where
    PTE: PrimInt + Unsigned,
{}

/// This represents a page table backed by a mapped physical page and provides mutable access to
/// the PTEs to [`crate::walker::PageWalkerMut`].
pub trait PageTableMut<PTE>: AsMut<[PTE]>
where
    PTE: PrimInt + Unsigned,
{}
