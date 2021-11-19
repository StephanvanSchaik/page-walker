//! This modules implements the [`PteAllocator`] struct which is a helper used to allocate the pages
//! and the underlying page tables for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PageFormat;

/// The [`PteAllocator`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// allocate pages and the underlying page tables for a given virtual address range. This is used
/// by the [`AddressSpace::allocate_range`] method.
///
/// [`AddressSpace::allocate_range`]: `super::super::AddressSpace::allocate_range`
pub struct PteAllocator<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// The page format.
    pub format: &'a PageFormat<'a>,
    /// The mask to set for pages.
    pub mask: Option<u64>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
    /// A marker for Mapper.
    pub mapper: PhantomData<Mapper>,
}

impl<'a, Mapper, Error> crate::PageWalkerMut<Mapper, Error> for PteAllocator<'a, Mapper, Error>
where
    Mapper: PageTableMapper<Error>,
{
    /// Allocates the page or page table for the current level as we are handling PTE holes. If the
    /// mask is set to None, then this function only allocates page tables.
    fn handle_pte_hole(&mut self, mapper: &mut Mapper, index: usize, _range: Range<usize>, pte: &mut u64) -> Result<(), Error> {
        let level = &self.format.levels[index];

        match index {
            0 => {
                if let Some(mask) = self.mask {
                    let page = mapper.alloc_page()?;

                    // Mark the page as present and set the page mask.
                    *pte = page | level.present_bit.1 | mask;
                }
            }
            _ => {
                let page_table = mapper.alloc_page()?;

                // Mark the page table as present, set the page table mask and ensure it is
                // **not** a huge page.
                *pte = page_table | level.present_bit.1 | level.page_table_mask |
                    ((level.huge_page_bit.0 ^ level.huge_page_bit.1) & level.huge_page_bit.0);
            }
        }

        Ok(())
    }
}
