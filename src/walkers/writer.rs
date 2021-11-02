//! This modules implements the [`PteWriter`] struct which is a helper used to modify the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;
use num_traits::{PrimInt, Unsigned};

/// The [`PteWriter`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// store the PTE for a given virtual address, which is used by the [`AddressSpace::write_pte`]
/// method.
///
/// [`AddressSpace::write_pte`]: `super::super::AddressSpace::write_pte`
pub struct PteWriter<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    pub mapper: &'a Mapper,
    /// The PTE to store.
    pub pte: PTE,
    /// A marker for PageTable.
    pub page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    pub page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteWriter<'a, PTE, PageTable, PageTableMut, Mapper, Error>
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
        if let PteType::Page(_) = pte_type {
            *pte = self.pte;
        }

        Ok(())
    }
}
