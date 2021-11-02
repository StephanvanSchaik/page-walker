//! This modules implements the [`PteAllocate`] struct which is a helper used to allocate the pages
//! and the underlying page tables for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PageFormat;
use num_traits::{PrimInt, Unsigned};

/// The [`PteMapper`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to map
/// a physical address range and allocate the underlying page tables for a given virtual address
/// range. This is used by the [`AddressSpace::map_range`] method.
pub(crate) struct PteMapper<'a, PTE, PageTable, PageTableMut, Mapper, Error>
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
    /// The mask to set for pages.
    pub(crate) mask: PTE,
    /// A marker for PageTable.
    pub(crate) page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub(crate) page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub(crate) error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteMapper<'a, PTE, PageTable, PageTableMut, Mapper, Error>
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

    /// Allocates the page or page table for the current level as we are handling PTE holes. If the
    /// mask is set to None, then this function only allocates page tables.
    fn handle_pte_hole(&mut self, index: usize, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        let level = &self.format.levels[index];

        match index {
            0 => {
                // Mark the page as present and set the page mask.
                *pte = level.present_bit.1 | self.mask;
                self.mask = self.mask + PTE::from(level.page_size()).unwrap_or(PTE::zero());
            }
            _ => {
                if let Some(page_table) = self.mapper.alloc_page() {
                    // Mark the page table as present, set the page table mask and ensure it is
                    // **not** a huge page.
                    *pte = page_table | level.present_bit.1 | level.page_table_mask |
                        level.huge_page_bit.0 ^ level.huge_page_bit.1;
                }
            }
        }

        Ok(())
    }
}
