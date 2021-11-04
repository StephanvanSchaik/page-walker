//! This modules implements the [`CopyFromWalker`] and [`CopyToWalker`] structs which are walkers used to copy data from and to a virtual address range.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};
use num_traits::{FromPrimitive, PrimInt, Unsigned};

/// The [`CopyFromWalker`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// copy data from a given a virtual address range.
///
/// This is used by the [`AddressSpace::copy_from`] method.
///
/// [`AddressSpace::copy_from`]: `super::super::AddressSpace::copy_from`
pub struct CopyFromWalker<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a Mapper,
    /// The offset within the buffer.
    pub offset: usize,
    /// Storage for the copied data.
    pub data: &'a mut [u8],
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalker<PTE, Error> for CopyFromWalker<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: PTE) -> Result<PTE, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Maps the page and copies the data to the buffer.
    fn handle_pte(&mut self, pte_type: PteType, range: Range<usize>, pte: &PTE) -> Result<(), Error> {
        let level = match pte_type {
            PteType::Page(level) => level,
            _ => return Ok(()),
        };

        let level = &self.format.levels[level];

        if !level.is_present(*pte) {
            return Err(Mapper::PAGE_NOT_PRESENT);
        }

        // Map the page.
        let page = self.mapper.map_page(*pte & self.format.physical_mask)?;

        // Get the page offset.
        let offset = range.start & (level.page_size() - 1);
        let page = &page[offset..];

        // Determine how many bytes to copy.
        let size = self.data.len().min(page.len());

        // Copy the bytes.
        self.data[self.offset..self.offset + size].copy_from_slice(&page[..size]);
        self.offset += size;

        Ok(())
    }
}

/// The [`CopyToWalker`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// copy data to a given a virtual address range.
///
/// This is used by the [`AddressSpace::copy_to`] method.
///
/// [`AddressSpace::copy_to`]: `super::super::AddressSpace::copy_to`
pub struct CopyToWalker<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a Mapper,
    /// The offset within the buffer.
    pub offset: usize,
    /// Storage for the data to copy.
    pub data: &'a [u8],
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalker<PTE, Error> for CopyToWalker<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: PTE) -> Result<PTE, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Maps the page and copies the data from the buffer.
    fn handle_pte(&mut self, pte_type: PteType, range: Range<usize>, pte: &PTE) -> Result<(), Error> {
        let level = match pte_type {
            PteType::Page(level) => level,
            _ => return Ok(()),
        };

        let level = &self.format.levels[level];

        if !level.is_present(*pte) {
            return Err(Mapper::PAGE_NOT_PRESENT);
        }

        // Map the page.
        let page = self.mapper.map_page_mut(*pte & self.format.physical_mask)?;

        // Get the page offset.
        let offset = range.start & (level.page_size() - 1);
        let page = &mut page[offset..];

        // Determine how many bytes to copy.
        let size = self.data.len().min(page.len());

        // Copy the bytes.
        page[..size].copy_from_slice(&self.data[self.offset..self.offset + size]);
        self.offset += size;

        Ok(())
    }
}
