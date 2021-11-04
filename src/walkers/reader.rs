//! This modules implements the [`PteReader`] struct which is a helper used to read the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;
use num_traits::{FromPrimitive, PrimInt, Unsigned};

/// The [`PteReader`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// retrieve the PTE for a given virtual address, which is used by the [`AddressSpace::read_pte`]
/// method.
///
/// [`AddressSpace::read_pte`]: `super::super::AddressSpace::read_pte`
pub struct PteReader<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a Mapper,
    /// Storage for the retrieved PTE.
    pub pte: Option<PTE>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalker<PTE, Error> for PteReader<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: PTE) -> Result<PTE, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Stores the PTE of the page, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &PTE) -> Result<(), Error> {
        if pte_type.is_page() {
            self.pte = Some(*pte);
        }

        Ok(())
    }
}
