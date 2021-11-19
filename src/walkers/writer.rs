//! This modules implements the [`PteWriter`] struct which is a helper used to modify the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;

/// The [`PteWriter`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// store the PTE for a given virtual address, which is used by the [`AddressSpace::write_pte`]
/// method.
///
/// [`AddressSpace::write_pte`]: `super::super::AddressSpace::write_pte`
pub struct PteWriter<Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The PTE to store.
    pub pte: u64,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<Mapper, Error> crate::PageWalkerMut<Mapper, Error> for PteWriter<Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Store the PTE, if the virtual address resolves to a page.
    fn handle_pte(&mut self, _mapper: &mut Mapper, pte_type: PteType, _range: Range<usize>, pte: &mut u64) -> Result<(), Error> {
        if let PteType::Page(_) = pte_type {
            *pte = self.pte;
        }

        Ok(())
    }
}
