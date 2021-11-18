//! This module provides the [`AddressSpace`] struct which provides an abstraction over a virtual
//! address space and provides methods to introspect and manage the virtual address space.

use core::marker::PhantomData;
use core::ops::Range;
use crate::PageFormat;
use crate::walkers::*;

/// The [`AddressSpace`] struct expects a type implementing this trait in order to map the page
/// tables while performing the various page table operations.
pub trait PageTableMapper<Error> {
    /// An `Error` constant indicating that the PTE was not found.
    const PTE_NOT_FOUND: Error;

    /// An `Error` constant indicating that a page was not present.
    const PAGE_NOT_PRESENT: Error;

    /// An `Error` constant indicating that a function has not been implemented.
    const NOT_IMPLEMENTED: Error;

    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: u64) -> Result<u64, Error>;

    /// Writes the PTE to the given physical address.
    fn write_pte(&mut self, phys_addr: u64, value: u64) -> Result<(), Error>;

    /// Reads the bytes from the given physical address.
    fn read_bytes(&self, _bytes: &mut [u8], _phys_addr: u64) -> Result<usize, Error> {
        Err(Self::NOT_IMPLEMENTED)
    }

    /// Writes the given bytes to the given physical address.
    fn write_bytes(&mut self, _phys_addr: u64, _bytes: &[u8]) -> Result<usize, Error> {
        Err(Self::NOT_IMPLEMENTED)
    }

    /// Allocates a physical page.
    fn alloc_page(&mut self) -> Result<u64, Error> {
        Err(Self::NOT_IMPLEMENTED)
    }

    /// Frees a physical page.
    fn free_page(&mut self, _pte: u64) {
    }
}

/// Abstracts a virtual address space.
pub struct AddressSpace<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The page table format describing the page table hierarchy for this virtual address space.
    format: PageFormat<'a>,

    /// The root address of the page table hierarchy.
    root: u64,

    /// The type implementing PageTableMapper.
    mapper: &'a mut Mapper,

    /// A marker for Error.
    error: core::marker::PhantomData<Error>,
}

impl<'a, Mapper, Error> AddressSpace<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Creates a new address space for the given page table format descripting the page table
    /// hierarchy, the page table mapper and the pointer to the root of the page table
    /// hierarchy.
    pub fn new(format: PageFormat<'a>, mapper: &'a mut Mapper, root: u64) -> Self {
        Self {
            format,
            mapper,
            root,
            error: PhantomData,
        }
    }

    /// Reads the PTE for the given the virtual address if the virtual address is valid.
    pub fn read_pte(&self, virt_addr: usize) -> Result<u64, Error> {
        let mut walker = PteReader {
            mapper: self.mapper,
            pte: None,
            error: PhantomData,
        };

        self.format.walk(self.root, virt_addr..virt_addr + 1, &mut walker)?;

        match walker.pte {
            Some(pte) => Ok(pte),
            _ => Err(Mapper::PTE_NOT_FOUND),
        }
    }

    /// Writes the PTE for the given virtual address if the virtual address is valid.
    pub fn write_pte(&mut self, virt_addr: usize, pte: u64) -> Result<(), Error> {
        let mut walker = PteWriter {
            mapper: self.mapper,
            pte,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, virt_addr..virt_addr + 1, &mut walker)?;

        Ok(())
    }

    /// Allocates pages and the underlying page tables for a given range in the virtual address
    /// space. The pages are protected using the given mask.
    pub fn allocate_range(&mut self, range: Range<usize>, mask: u64) -> Result<(), Error> {
        let mut walker = PteAllocator {
            mapper: self.mapper,
            mask: Some(mask),
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Maps the given range in the virtual address space range to the given physical address
    /// offset and mask. Allocates the underlying page tables if they are missing. This is useful
    /// for memory-mapped I/O.
    pub fn map_range(&mut self, range: Range<usize>, mask: u64) -> Result<(), Error> {
        let mut walker = PteMapper {
            mapper: self.mapper,
            mask,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Changes the protection flags of the given range in the virtual address space. The first
    /// mask specifies the full mask to clear the bits. The second mask specifies the bits that
    /// should be set.
    pub fn protect_range(&mut self, range: Range<usize>, mask: (u64, u64)) -> Result<(), Error> {
        let mut walker = PteProtector {
            mapper: self.mapper,
            mask,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Frees the pages for the given range in the virtual address space. If the underlying page
    /// tables have been cleared, then this function also free the underlying page tables.
    pub fn free_range(&mut self, range: Range<usize>) -> Result<(), Error> {
        let flags = PteRemovalFlags::all();

        let mut walker = PteRemover {
            mapper: self.mapper,
            flags,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Unmaps the pages for the given range in the virtual address space without freeing the
    /// underlying pages. This is useful for memory-mapped I/O.
    pub fn unmap_range(&mut self, range: Range<usize>) -> Result<(), Error> {
        let flags = PteRemovalFlags::empty();

        let mut walker = PteRemover {
            mapper: self.mapper,
            flags,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Copies bytes starting at the given address into the given buffer.
    pub fn copy_from(&mut self, data: &mut [u8], address: usize) -> Result<(), Error> {
        let range = address..address + data.len();

        let mut walker = CopyFromWalker {
            mapper: self.mapper,
            offset: 0,
            data,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk(self.root, range, &mut walker)?;

        Ok(())
    }

    /// Copies bytes from the given buffer to the given address.
    pub fn copy_to(&mut self, address: usize, data: &[u8]) -> Result<(), Error> {
        let range = address..address + data.len();

        let mut walker = CopyToWalker {
            mapper: self.mapper,
            offset: 0,
            data,
            format: &self.format,
            error: PhantomData,
        };

        self.format.walk(self.root, range, &mut walker)?;

        Ok(())
    }
}
