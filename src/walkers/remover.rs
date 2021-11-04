//! This modules implements the [`PteRemover`] struct which is a helper used to remove the pages and
//! the underlying page tables for a given range of virtual addresses.

use bitflags::bitflags;
use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};
use num_traits::{FromPrimitive, PrimInt, Unsigned};

bitflags! {
    /// Flags to configure the behavior of the `[PteRemover`] walker.
    pub struct PteRemovalFlags: u32 {
        /// Free the pages.
        const FREE_PAGES       = 1 << 0;

        /// Free the page tables if fully cleared.
        const FREE_PAGE_TABLES = 1 << 1;
    }
}

/// The [`PteRemover`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// remove pages and the underlying page tables for a given virtual address range. This is used by
/// the [`AddressSpace::unmap_range`] and [`AddressSpace::free_range`] methods.
///
/// [`AddressSpace::unmap_range`]: `super::super::AddressSpace::unmap_range`
/// [`AddressSpace::free_range`]: `super::super::AddressSpace::free_range`
pub struct PteRemover<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a mut Mapper,
    /// Flags to configure the behavior.
    pub flags: PteRemovalFlags,
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalkerMut<PTE, Error> for PteRemover<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: PTE) -> Result<PTE, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Writes the PTE to the given physical address.
    fn write_pte(&mut self, phys_addr: PTE, value: PTE) -> Result<(), Error> {
        self.mapper.write_pte(phys_addr, value)
    }

    /// Frees the page if the PTE points to a present page and zeroes the PTE afterwards.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> { 
        let physical_mask = self.format.physical_mask;

        if let PteType::Page(level) = pte_type {
            let level = &self.format.levels[level];

            if level.is_present(*pte) {
                // Free the page and mark the PTE as non-present.
                if self.flags.contains(PteRemovalFlags::FREE_PAGES) {
                    self.mapper.free_page(physical_mask & *pte);
                }

                *pte = PTE::zero();
            }
        }

        Ok(())
    }

    /// Maps in the page table to check if all entries have been cleared. If so, this function
    /// frees the page table.
    fn handle_post_pte(&mut self, index: usize, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        let level = &self.format.levels[index];
        let physical_mask = self.format.physical_mask;
        let phys_addr = physical_mask & *pte;

        // Check if all entries have been cleared.
        for i in 0..level.entries() {
            let offset: PTE = PTE::from_usize(i * core::mem::size_of::<PTE>()).unwrap();

            if self.read_pte(phys_addr + offset)? != PTE::zero() {
                return Ok(());
            }
        }

        if self.flags.contains(PteRemovalFlags::FREE_PAGE_TABLES) {
            self.mapper.free_page(physical_mask & *pte);
            *pte = PTE::zero();
        }

        Ok(())
    }
}
