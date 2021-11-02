//! This modules implements the [`PteProtect`] struct which is a helper used to change the
//! protection flags for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};
use num_traits::{PrimInt, Unsigned};

/// The [`PteProtect`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// change the protection flags of a given virtual address range, which is used by the
/// [`AddressSpace::protect_range`] method.
pub(crate) struct PteProtect<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    pub(crate) mapper: &'a Mapper,
    /// The protection flags that should be set. The first mask is the mask of bits that should be
    /// cleared. The second mask is the mask of bits that should be set.
    pub(crate) mask: (PTE, PTE),
    /// The page format.
    pub(crate) format: &'a PageFormat<'a, PTE>,
    /// A marker for PageTable.
    pub(crate) page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub(crate) page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub(crate) error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteProtect<'a, PTE, PageTable, PageTableMut, Mapper, Error>
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

    /// Store the PTE, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        if let PteType::Page(level) = pte_type {
            let physical_mask = self.format.physical_mask;
            let level = &self.format.levels[level];

            // Ensure the mask does not modify the physical address bits, the huge page bits or the
            // present bits.
            let clear_mask = self.mask.0 &
                !(physical_mask | level.huge_page_bit.0 | level.present_bit.0);
            let set_mask   = self.mask.1 &
                !(physical_mask | level.huge_page_bit.0 | level.present_bit.0);

            *pte = (*pte & !clear_mask) | set_mask;
        }

        Ok(())
    }
}
