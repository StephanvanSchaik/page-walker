//! This modules implements the [`PteRemover`] struct which is a helper used to remove the pages and
//! the underlying page tables for a given range of virtual addresses.

use bitflags::bitflags;
use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};
use num_traits::{PrimInt, Unsigned};

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
pub struct PteRemover<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    pub mapper: &'a mut Mapper,
    /// Flags to configure the behavior.
    pub flags: PteRemovalFlags,
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// A marker for PageTable.
    pub page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteRemover<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Uses the page table mapper to map the page table backing the physical address.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTableMut, Error> {
        self.mapper.map_table_mut(phys_addr)
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

        // Map in the page table.
        let mut page_table = self.map_table(physical_mask & *pte)?;
        let entries = page_table.as_mut();

        // Check if all entries have been cleared.
        for i in 0..level.entries() {
            if entries[i] != PTE::zero() {
                return Ok(());
            }
        }

        // Free the page table.
        drop(page_table);

        if self.flags.contains(PteRemovalFlags::FREE_PAGE_TABLES) {
            self.mapper.free_page(physical_mask & *pte);
        }

        Ok(())
    }
}
