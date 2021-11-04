//! This modules implements the [`PteWriter`] struct which is a helper used to modify the PTE
//! pointing to the physical page for a given virtual address, if the virtual address is mapped.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PteType;
use num_traits::{FromPrimitive, PrimInt, Unsigned};

/// The [`PteWriter`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// store the PTE for a given virtual address, which is used by the [`AddressSpace::write_pte`]
/// method.
///
/// [`AddressSpace::write_pte`]: `super::super::AddressSpace::write_pte`
pub struct PteWriter<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a mut Mapper,
    /// The PTE to store.
    pub pte: PTE,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalkerMut<PTE, Error> for PteWriter<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// Reads the PTE at the given physical address.
    fn read_pte(&self, phys_addr: PTE) -> Result<PTE, Error> {
        self.mapper.read_pte(phys_addr)
    }

    /// Writes the PTE to the given physical address.
    fn write_pte(&mut self, phys_addr: PTE, value: PTE) -> Result<(), Error> {
        self.mapper.write_pte(phys_addr, value)
    }

    /// Store the PTE, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        if let PteType::Page(_) = pte_type {
            *pte = self.pte;
        }

        Ok(())
    }
}
