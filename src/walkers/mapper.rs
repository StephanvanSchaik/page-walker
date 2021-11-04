//! This modules implements the [`PteMapper`] struct which is a helper used to map a physical
//! address range and allocate the underlying page tables for a given virtual address range.

use core::marker::PhantomData;
use core::ops::Range;
use crate::address_space::PageTableMapper;
use crate::PageFormat;
use num_traits::{FromPrimitive, PrimInt, Unsigned};

/// The [`PteMapper`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to map
/// a physical address range and allocate the underlying page tables for a given virtual address
/// range. This is used by the [`AddressSpace::map_range`] method.
///
/// [`AddressSpace::map_range`]: `super::super::AddressSpace::map_range`
pub struct PteMapper<'a, PTE, Mapper, Error>
where
    PTE: FromPrimitive + PrimInt + Unsigned,
    Mapper: PageTableMapper<PTE, Error>,
{
    /// The page table mapper.
    pub mapper: &'a mut Mapper,
    /// The page format.
    pub format: &'a PageFormat<'a, PTE>,
    /// The mask to set for pages.
    pub mask: PTE,
    /// A marker for Error.
    pub error: PhantomData<Error>,
}

impl<'a, PTE, Mapper, Error> crate::PageWalkerMut<PTE, Error> for PteMapper<'a, PTE, Mapper, Error>
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
                // Mark the page as present and set the page mask.
                *pte = level.present_bit.1 | self.mask;
                self.mask = self.mask + PTE::from(level.page_size()).unwrap_or(PTE::zero());
            }
            _ => {
                let page_table = self.mapper.alloc_page()?;

                // Mark the page table as present, set the page table mask and ensure it is
                // **not** a huge page.
                *pte = page_table | level.present_bit.1 | level.page_table_mask |
                    level.huge_page_bit.0 ^ level.huge_page_bit.1;
            }
        }

        Ok(())
    }
}
