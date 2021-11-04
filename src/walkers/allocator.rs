//! This modules implements the [`PteAllocator`] struct which is a helper used to allocate the pages
//! and the underlying page tables for a given range of virtual addresses.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PageFormat;
use num_traits::{FromPrimitive, PrimInt, Unsigned};

/// The [`PteAllocator`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// allocate pages and the underlying page tables for a given virtual address range. This is used
/// by the [`AddressSpace::allocate_range`] method.
///
/// [`AddressSpace::allocate_range`]: `super::super::AddressSpace::allocate_range`
pub struct PteAllocator<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a mut Mapper,
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// The mask to set for pages.
    pub mask: Option<PTE>,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalkerMut<PTE, Error> for PteAllocator<'a, PTE, Mapper, Error>
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

    /// Allocates the page or page table for the current level as we are handling PTE holes. If the
    /// mask is set to None, then this function only allocates page tables.
    fn handle_pte_hole(&mut self, index: usize, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        let level = &self.format.levels[index];

        match index {
            0 => {
                if let Some(mask) = self.mask {
                    let page = self.mapper.alloc_page()?;

                    // Mark the page as present and set the page mask.
                    *pte = page | level.present_bit.1 | mask;
                }
            }
            _ => {
                let page_table = self.mapper.alloc_page()?;

                // Mark the page table as present, set the page table mask and ensure it is
                // **not** a huge page.
                *pte = page_table | level.present_bit.1 | level.page_table_mask |
                    ((level.huge_page_bit.0 ^ level.huge_page_bit.1) & level.huge_page_bit.0);
            }
        }

        Ok(())
    }
}
