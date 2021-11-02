//! This modules implements the [`PteReader`] struct which is a helper used to read the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;
use num_traits::{PrimInt, Unsigned};

/// The [`PteReader`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// retrieve the PTE for a given virtual address, which is used by the [`AddressSpace::read_pte`]
/// method.
pub(crate) struct PteReader<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    pub(crate) mapper: &'a Mapper,
    /// Storage for the retrieved PTE.
    pub(crate) pte: Option<PTE>,
    /// A marker for PageTable.
    pub(crate) page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub(crate) page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub(crate) error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalker<PTE, PageTable, Error> for PteReader<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Uses the page table mapper to map the page table backing the physical address.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTable, Error> {
        self.mapper.map_table(phys_addr)
    }

    /// Stores the PTE of the page, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &PTE) -> Result<(), Error> {
        if pte_type.is_page() {
            self.pte = Some(*pte);
        }

        Ok(())
    }
}
