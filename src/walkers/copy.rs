//! This modules implements the [`CopyFromWalker`] and [`CopyToWalker`] structs which are walkers used to copy data from and to a virtual address range.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};

/// The [`CopyFromWalker`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// copy data from a given a virtual address range.
///
/// This is used by the [`AddressSpace::copy_from`] method.
///
/// [`AddressSpace::copy_from`]: `super::super::AddressSpace::copy_from`
pub struct CopyFromWalker<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The offset within the buffer.
    pub offset: usize,
    /// Storage for the copied data.
    pub data: &'a mut [u8],
    /// The page format.
    pub format: &'a PageFormat<'a>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<'a, Mapper, Error> crate::PageWalker<Mapper, Error> for CopyFromWalker<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Maps the page and copies the data to the buffer.
    fn handle_pte(&mut self, mapper: &Mapper, pte_type: PteType, range: Range<usize>, pte: &u64) -> Result<(), Error> {
        let level = match pte_type {
            PteType::Page(level) => level,
            _ => return Ok(()),
        };

        let level = &self.format.levels[level];

        if !level.is_present(*pte) {
            return Err(Mapper::PAGE_NOT_PRESENT);
        }

        // Get the physical address of the page.
        let phys_addr = *pte & self.format.physical_mask;

        // Get the page offset.
        let offset = (range.start & (level.page_size() - 1)) as u64;

        // Determine how many bytes to copy.
        let size = (self.data.len() - self.offset).min(level.page_size());

        // Copy the bytes.
        mapper.read_bytes(&mut self.data[self.offset..self.offset + size], phys_addr + offset)?;
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
pub struct CopyToWalker<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The offset within the buffer.
    pub offset: usize,
    /// Storage for the data to copy.
    pub data: &'a [u8],
    /// The page format.
    pub format: &'a PageFormat<'a>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<'a, Mapper, Error> crate::PageWalkerMut<Mapper, Error> for CopyToWalker<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Maps the page and copies the data from the buffer.
    fn handle_pte(&mut self, mapper: &mut Mapper, pte_type: PteType, range: Range<usize>, pte: &mut u64) -> Result<(), Error> {
        let level = match pte_type {
            PteType::Page(level) => level,
            _ => return Ok(()),
        };

        let level = &self.format.levels[level];

        if !level.is_present(*pte) {
            return Err(Mapper::PAGE_NOT_PRESENT);
        }

        // Get the physical address of the page.
        let phys_addr = *pte & self.format.physical_mask;

        // Get the page offset.
        let offset = (range.start & (level.page_size() - 1)) as u64;

        // Determine how many bytes to copy.
        let size = (self.data.len() - self.offset).min(level.page_size());

        // Copy the bytes.
        mapper.write_bytes(phys_addr + offset, &self.data[self.offset..self.offset + size])?;
        self.offset += size;

        Ok(())
    }
}
