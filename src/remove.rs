//! This modules implements the [`PteRemove`] struct which is a helper used to remove the pages and
//! the underlying page tables for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};
use num_traits::{PrimInt, Unsigned};

/// The [`PteRemove`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// remove pages and the underlying page tables for a given virtual address range. This is used by
/// the [`AddressSpace::remove_range`] method.
pub(crate) struct PteRemove<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    pub(crate) mapper: &'a Mapper,
    /// The page format.
    pub(crate) format: &'a PageFormat<'a, PTE>,
    /// A marker for PageTable.
    pub(crate) page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub(crate) page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub(crate) error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteRemove<'a, PTE, PageTable, PageTableMut, Mapper, Error>
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
        self.mapper.free_page(physical_mask & *pte);

        if let PteType::Page(level) = pte_type {
            let level = &self.format.levels[level];

            if level.is_present(*pte) {
                // Free the page and mark the PTE as non-present.
                let physical_mask = self.format.physical_mask;
                self.mapper.free_page(physical_mask & *pte);
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
        self.mapper.free_page(physical_mask & *pte);

        Ok(())
    }
}
