//! This module provides the [`AddressSpace`] struct which provides an abstraction over a virtual
//! address space and provides methods to introspect and manage the virtual address space.

use core::marker::PhantomData;
use core::ops::Range;
use crate::allocator::PteAllocator;
use crate::PageFormat;
use crate::protector::PteProtector;
use crate::reader::PteReader;
use crate::remover::{PteRemover, PteRemovalFlags};
use crate::writer::PteWriter;
use num_traits::{PrimInt, Unsigned};

/// The [`AddressSpace`] struct expects a type implementing this trait in order to map the page
/// tables while performing the various page table operations.
pub trait PageTableMapper<PTE, PageTable, PageTableMut, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
{
    /// An `Error` constant indicating that the PTE was not found.
    const PTE_NOT_FOUND: Error;

    /// Given the physical address, maps in the physical page backing the page table. To unmap the
    /// page upon use, the type implementing [`crate::table::PageTable`] must implement
    /// [`core::ops::Drop`] semantics.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTable, Error>;

    /// Given the physical address, maps in the physical page backing the page table. To unmap the
    /// page upon use, the type implementing [`crate::table::PageTableMut`] must implement
    /// [`core::ops::Drop`] semantics.
    fn map_table_mut(&self, phys_addr: PTE) -> Result<PageTableMut, Error>;

    /// Allocates a physical page.
    fn alloc_page(&self) -> Option<PTE> {
        None
    }

    /// Frees a physical page.
    fn free_page(&self, _pte: PTE) {
    }
}

/// Abstracts a virtual address space.
pub struct AddressSpace<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table format describing the page table hierarchy for this virtual address space.
    format: PageFormat<'a, PTE>,

    /// The root address of the page table hierarchy.
    root: PTE,

    /// The type implementing PageTableMapper.
    mapper: Mapper,

    /// A marker for PageTable.
    page_table: core::marker::PhantomData<PageTable>,

    /// A marker for PageTableMut.
    page_table_mut: core::marker::PhantomData<PageTableMut>,

    /// A marker for Error.
    error: core::marker::PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> AddressSpace<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Creates a new address space for the given page table format descripting the page table
    /// hierarchy, the page table mapper and the pointer to the root of the page table
    /// hierarchy.
    pub fn new(format: PageFormat<'a, PTE>, mapper: Mapper, root: PTE) -> Self {
        Self {
            format,
            mapper,
            root,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        }
    }

    /// Reads the PTE for the given the virtual address if the virtual address is valid.
    pub fn read_pte(&self, virt_addr: usize) -> Result<PTE, Error> {
        let mut walker = PteReader {
            mapper: &self.mapper,
            pte: None,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk(self.root, virt_addr..virt_addr + 1, &mut walker)?;

        match walker.pte {
            Some(pte) => Ok(pte),
            _ => Err(Mapper::PTE_NOT_FOUND),
        }
    }

    /// Writes the PTE for the given virtual address if the virtual address is valid.
    pub fn write_pte(&self, virt_addr: usize, pte: PTE) -> Result<(), Error> {
        let mut walker = PteWriter {
            mapper: &self.mapper,
            pte,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, virt_addr..virt_addr + 1, &mut walker)?;

        Ok(())
    }

    /// Allocates pages and the underlying page tables for a given range in the virtual address
    /// space range.
    pub fn allocate_range(&self, range: Range<usize>, mask: PTE) -> Result<(), Error> {
        let mut walker = PteAllocator {
            mapper: &self.mapper,
            mask: Some(mask),
            format: &self.format,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Changes the protection flags of the given range in the virtual address space. The first
    /// mask specifies the full mask to clear the bits. The second mask specifies the bits that
    /// should be set.
    pub fn protect_range(&self, range: Range<usize>, mask: (PTE, PTE)) -> Result<(), Error> {
        let mut walker = PteProtector {
            mapper: &self.mapper,
            mask,
            format: &self.format,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Frees the pages for the given range in the virtual address space. If the underlying page
    /// tables have been cleared, then this function also free the underlying page tables.
    pub fn free_range(&self, range: Range<usize>) -> Result<(), Error> {
        let flags = PteRemovalFlags::empty();

        let mut walker = PteRemover {
            mapper: &self.mapper,
            flags,
            format: &self.format,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Unmaps the pages for the given range in the virtual address space without freeing the
    /// underlying pages. This is useful for memory-mapped I/O.
    pub fn unmap_range(&self, range: Range<usize>) -> Result<(), Error> {
        let flags = PteRemovalFlags::all();

        let mut walker = PteRemover {
            mapper: &self.mapper,
            flags,
            format: &self.format,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }
}
