//! This modules implements the [`PteReader`] struct which is a helper used to read the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;

/// The [`PteReader`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// retrieve the PTE for a given virtual address, which is used by the [`AddressSpace::read_pte`]
/// method.
///
/// [`AddressSpace::read_pte`]: `super::super::AddressSpace::read_pte`
pub struct PteReader<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The page table mapper.
    pub mapper: &'a Mapper,
    /// Storage for the retrieved PTE.
    pub pte: Option<u64>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, Mapper, Error> crate::PageWalker<Error> for PteReader<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: u64) -> Result<u64, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Stores the PTE of the page, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &u64) -> Result<(), Error> {
        if pte_type.is_page() {
            self.pte = Some(*pte);
        }

        Ok(())
    }
}
