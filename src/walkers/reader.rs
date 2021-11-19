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
pub struct PteReader<Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Storage for the retrieved PTE.
    pub pte: Option<u64>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<Mapper, Error> crate::PageWalker<Mapper, Error> for PteReader<Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Stores the PTE of the page, if the virtual address resolves to a page.
    fn handle_pte(&mut self, _mapper: &Mapper, pte_type: PteType, _range: Range<usize>, pte: &u64) -> Result<(), Error> {
        if pte_type.is_page() {
            self.pte = Some(*pte);
        }

        Ok(())
    }
}
