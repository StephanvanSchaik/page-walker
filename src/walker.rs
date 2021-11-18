//! This module provides the [`PageWalker`] and [`PageWalkerMut`] traits that are used by
//! [`crate::format::PageFormat::walk`] and [`crate::format::PageFormat::walk_mut`] to invoke
//! user-specified callbacks during a page table walk, allowing the user to interact with the page
//! tables of an address space in a generic way.

use core::ops::Range;

/// The PTE can either be a page or page table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PteType {
    /// The PTE refers to a physical page.
    Page(usize),
    /// The PTE refers to another page table.
    PageTable(usize),
}

impl PteType {
    /// Returns `true` if the [`PteType`] is a page and `false` otherwise.
    pub fn is_page(&self) -> bool {
        match self {
            PteType::Page(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the [`PteType`] is a page table and `false` otherwise.
    pub fn is_page_table(&self) -> bool {
        match self {
            PteType::PageTable(_) => true,
            _ => false,
        }
    }

    /// Extracts the level at which the PTE is found. The level is a monotonicly increasing number
    /// that starts at zero for the leaf page table and where the maximum number is the root page
    /// table.
    pub fn level(&self) -> usize {
        match self {
            Self::Page(level) => *level,
            Self::PageTable(level) => *level,
        }
    }

    /// Returns whether the current PTE refers to a huge page, i.e. it checks whether the page type
    /// is a page and the level is non-zero. Returns `true` if it is a huge page and `false`
    /// otherwise.
    pub fn is_huge_page(&self) -> bool {
        match self {
            Self::Page(level) if *level != 0 => true,
            _ => false,
        }
    }
}

/// The [`crate::format::PageFormat::walk`] function expects a type that implements this trait to
/// invoke the appropriate user callbacks, such that the user can provide an implementation for
/// interacting with the various PTEs during the page table walk. For the mutable version, see
/// [`crate::format::PageFormat::walk_mut`] and [`PageWalkerMut`].
pub trait PageWalker<Error> {
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: u64) -> Result<u64, Error>;

    /// This callback handles the current PTE unconditionally and is given the [`PteType`], the
    /// virtual address range and an immutable reference to the PTE. The implementation of this
    /// callback is optional.
    fn handle_pte(
        &mut self,
        _page_type: PteType,
        _range: Range<usize>,
        _pte: &u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// This callback handles a PTE hole, i.e. a PTE that is not marked as present, and is given
    /// the level, the virtual address range and an immutable reference to the PTE. The
    /// implementation of this callback is optional.
    fn handle_pte_hole(
        &mut self,
        _level: usize,
        _range: Range<usize>,
        _pte: &u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// This callback handles the PTE of a page table after recursing the page table hierarchy, and
    /// is given the level, the virtual address and an immutable reference to the PTE. The
    /// implementation of this callback is optional.
    fn handle_post_pte(
        &mut self,
        _level: usize,
        _range: Range<usize>,
        _pte: &u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}

/// The [`crate::format::PageFormat::walk_mut`] function expects a type that implements this trait
/// to invoke the appropriate user callbacks, such that the user can provide an implementation for
/// interacting with the various PTEs during the page table walk. For the immutable version, see
/// [`crate::format::PageFormat::walk`] and [`PageWalker`].
pub trait PageWalkerMut<Error> {
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: u64) -> Result<u64, Error>;

    /// Writes the PTE to the given physical address.
    fn write_pte(&mut self, phys_addr: u64, value: u64) -> Result<(), Error>;

    /// This callback handles the current PTE unconditionally and is given the [`PteType`], the
    /// virtual address range and a mutable reference to the PTE. The implementation of this
    /// callback is optional.
    fn handle_pte(
        &mut self,
        _page_type: PteType,
        _range: Range<usize>,
        _pte: &mut u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// This callback handles a PTE hole, i.e. a PTE that is not marked as present, and is given
    /// the level, the virtual address range and a mutable reference to the PTE. The
    /// implementation of this callback is optional.
    fn handle_pte_hole(
        &mut self,
        _level: usize,
        _range: Range<usize>,
        _pte: &mut u64,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// This callback handles the PTE of a page table after recursing the page table hierarchy, and
    /// is given the level, the virtual address and a mutable reference to the PTE. The
    /// implementation of this callback is optional.
    fn handle_post_pte(
        &mut self,
        _level: usize,
        _range: Range<usize>,
        _pte: &mut u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}
