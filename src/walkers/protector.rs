//! This modules implements the [`PteProtector`] struct which is a helper used to change the
//! protection flags for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::{PageFormat, PteType};

/// The [`PteProtector`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// change the protection flags of a given virtual address range. This function is used by the
/// [`AddressSpace::protect_range`] method.
///
/// [`AddressSpace::protect_range`]: `super::super::AddressSpace::protect_range`
pub struct PteProtector<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The protection flags that should be set. The first mask is the mask of bits that should be
    /// cleared. The second mask is the mask of bits that should be set.
    pub mask: (u64, u64),
    /// The page format.
    pub format: &'a PageFormat<'a>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<'a, Mapper, Error> crate::PageWalkerMut<Mapper, Error> for PteProtector<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Checks if the PTE points to a page that is present, and changes the protection flags if so.
    fn handle_pte(&mut self, _mapper: &mut Mapper, pte_type: PteType, _range: Range<usize>, pte: &mut u64) -> Result<(), Error> {
        let physical_mask = self.format.physical_mask;

        if let PteType::Page(level) = pte_type {
            let level = &self.format.levels[level];

            if level.is_present(*pte) {
                // Ensure the mask does not modify the physical address bits, the huge page bits or the
                // present bits.
                let clear_mask = self.mask.0 &
                    !(physical_mask | level.huge_page_bit.0 | level.present_bit.0);
                let set_mask   = self.mask.1 &
                    !(physical_mask | level.huge_page_bit.0 | level.present_bit.0);

                *pte = (*pte & !clear_mask) | set_mask;
            }
        }

        Ok(())
    }
}
