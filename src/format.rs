//! This module provides the [`PageFormat`] struct that is used to describe the page table
//! hierarchy.

use core::ops::Range;
use crate::level::PageLevel;
use crate::walker::PteType;

/// Describes the page format of the page hierarchy and the mask of bits in the PTE that refer to
/// the actual physical address and are not used for metadata.
#[derive(Clone, Debug)]
pub struct PageFormat<'a> {
    /// Describes the page table hierarchy as a slice of [`crate::level::PageLevel`] structs that
    /// each describe a single level in this hierarchy, where the level at index zero is the leaf
    /// node and the last page level is the root.
    pub levels: &'a [PageLevel],

    /// The physical mask of bits that refer to an actual physical address and are not used for PTE
    /// metadata.
    pub physical_mask: u64,

    /// The size of a page table entry (PTE) in bytes.
    pub pte_size: usize,
}

impl<'a> PageFormat<'a> {
    /// Calculates the full virtual address mask by setting all the bits for each page level and
    /// finding the largest mask. This is used by the [`PageFormat::sign_extend`] method to
    /// determine the sign bit.
    pub fn virtual_mask(&self) -> usize {
        self.levels
            .iter()
            .map(|level| level.mask() | level.page_size() - 1)
            .max()
            .unwrap()
    }

    /// Sign extends a given virtual address by extending the sign bit into the unused upper bits
    /// of the virtual address.
    pub fn sign_extend(&self, address: usize) -> usize {
        let sign_bit = 1 << self.virtual_mask().trailing_ones() - 1;

        if address & sign_bit == sign_bit {
            // Invert the virtual mask and mask it with the address to sign extend the address.
            !self.virtual_mask() | address
        } else {
            address
        }
    }

    /// This is a recursive helper function used to traverse the page table hierarchy for a given
    /// virtual address range and the given physical address of the page table for the current page
    /// table level. It invokes the appropriate user callbacks in [`crate::walker::PageWalker`],
    /// while traversing the page tables.
    fn do_walk<PageWalker, Mapper, Error>(
        &self,
        phys_addr: u64,
        mut index: usize,
        range: Range<usize>,
        walker: &mut PageWalker,
        mapper: &Mapper,
    ) -> Result<(), Error>
    where
        PageWalker: crate::walker::PageWalker<Mapper, Error>,
        Mapper: crate::address_space::PageTableMapper<Error>,
    {
        // Ensure that the index is valid.
        if index >= self.levels.len() {
            index = self.levels.len() - 1;
        }

        let level = &self.levels[index];

        // Split up the range by page boundaries, such that we have a range for each page that is
        // inclusive of the original range. For instance, the range 0x0000..0x1fff spans two 4K
        // pages, so this iterator would return 0x0000..0x0fff and 0x1000..0x1fff. We also make
        // sure that the page ranges are sign extended where appropriate. In addition, calculate
        // the PTE index.
        let page_ranges = (level.pte_index(range.start)..=level.pte_index(range.end))
            .scan(self.sign_extend(range.start), |state, pte_index| {
                let page_range = *state..level.end(*state).min(range.end);
                *state = self.sign_extend(level.end(*state) + 1);

                Some((pte_index, page_range))
            });

        for (pte_index, page_range) in page_ranges {
            // Get the PTE index for this page range, and then index into the page table to get the
            // corresponding PTE.
            let offset = (pte_index * self.pte_size) as u64;
            let pte = mapper.read_pte(phys_addr + offset)?;

            // Determine whether the PTE refers to a page or a page table. That is, it is a page if
            // we are at a leaf page table or if the PTE refers to a huge page. Otherwise, it is a
            // page table.
            let page_type = match index == 0 || level.is_huge_page(pte) {
                true => PteType::Page(index),
                _    => PteType::PageTable(index),
            };

            // Invoke the user callback to handle this PTE.
            walker.handle_pte(mapper, page_type, page_range.clone(), &pte)?;

            // Invoke the user callback to handle this PTE hole, i.e. when the PTE is not marked as
            // present.
            if !level.is_present(pte) {
                walker.handle_pte_hole(mapper, index, page_range.clone(), &pte)?;
            }

            // If the user did not decide to unmap this page, then we are done with this PTE and
            // can resume to the next one.
            if index == 0 || level.is_huge_page(pte) {
                continue;
            }

            // At this point we are dealing with a normal page table. Extract the physical address
            // from the current PTE, and recurse the page table hierarchy.
            let phys_addr = pte & self.physical_mask;
            self.do_walk(phys_addr, index - 1, page_range.clone(), walker, mapper)?;

            // Provide an opportunity to the user to handle the PTE of the page table upon
            // recursion. For instance, to free the page table.
            walker.handle_post_pte(mapper, index, page_range, &pte)?;
        }

        Ok(())
    }

    /// This is a recursive function used to traverse the page table hierarchy for a given virtual
    /// address range and the given physical address of the root page table of the page table
    /// hierarchy. It invokes the appropriate user callbacks in [`crate::walker::PageWalker`],
    /// while traversing the page tables.
    pub fn walk<PageWalker, Mapper, Error>(
        &self,
        phys_addr: u64,
        range: Range<usize>,
        walker: &mut PageWalker,
        mapper: &Mapper,
    ) -> Result<(), Error>
    where
        PageWalker: crate::walker::PageWalker<Mapper, Error>,
        Mapper: crate::address_space::PageTableMapper<Error>,
    {
        self.do_walk(phys_addr, self.levels.len() - 1, range, walker, mapper)
    }

    /// This is a recursive helper function used to traverse the page table hierarchy for a given
    /// virtual address range and the given physical address of the page table for the current page
    /// table level. It invokes the appropriate user callbacks in [`crate::walker::PageWalkerMut`],
    /// while traversing the page tables.
    fn do_walk_mut<PageWalkerMut, Mapper, Error>(
        &self,
        phys_addr: u64,
        mut index: usize,
        range: Range<usize>,
        walker: &mut PageWalkerMut,
        mapper: &mut Mapper,
    ) -> Result<(), Error>
    where
        PageWalkerMut: crate::walker::PageWalkerMut<Mapper, Error>,
        Mapper: crate::address_space::PageTableMapper<Error>,
    {
        // Ensure that the index is valid.
        if index >= self.levels.len() {
            index = self.levels.len() - 1;
        }

        let level = &self.levels[index];

        // Split up the range by page boundaries, such that we have a range for each page that is
        // inclusive of the original range. For instance, the range 0x0000..0x1fff spans two 4K
        // pages, so this iterator would return 0x0000..0x0fff and 0x1000..0x1fff. We also make
        // sure that the page ranges are sign extended where appropriate. In addition, calculate
        // the PTE index.
        let page_ranges = (level.pte_index(range.start)..=level.pte_index(range.end))
            .scan(self.sign_extend(range.start), |state, pte_index| {
                let page_range = *state..level.end(*state).min(range.end);
                *state = self.sign_extend(level.end(*state) + 1);

                Some((pte_index, page_range))
            });

        for (pte_index, page_range) in page_ranges {
            // Get the PTE index for this page range, and then index into the page table to get the
            // corresponding PTE.
            let offset = (pte_index * self.pte_size) as u64;
            let mut pte = mapper.read_pte(phys_addr + offset)?;

            // Determine whether the PTE refers to a page or a page table. That is, it is a page if
            // we are at a leaf page table or if the PTE refers to a huge page. Otherwise, it is a
            // page table.
            let page_type = match index == 0 || level.is_huge_page(pte) {
                true => PteType::Page(index),
                _    => PteType::PageTable(index),
            };

            // Invoke the user callback to handle this PTE.
            walker.handle_pte(mapper, page_type, page_range.clone(), &mut pte)?;

            // Invoke the user callback to handle this PTE hole, i.e. when the PTE is not marked as
            // present.
            if !level.is_present(pte) {
                walker.handle_pte_hole(mapper, index, page_range.clone(), &mut pte)?;
            }

            // If the user did not decide to unmap this page, then we are done with this PTE and
            // can resume to the next one.
            mapper.write_pte(phys_addr + offset, pte)?;

            if index == 0 || level.is_huge_page(pte) {
                continue;
            }

            // At this point we are dealing with a normal page table. Extract the physical address
            // from the current PTE, and recurse the page table hierarchy.
            let phys_addr = pte & self.physical_mask;
            self.do_walk_mut(phys_addr, index - 1, page_range.clone(), walker, mapper)?;

            // Provide an opportunity to the user to handle the PTE of the page table upon
            // recursion. For instance, to free the page table.
            walker.handle_post_pte(mapper, index, page_range, &mut pte)?;
            mapper.write_pte(phys_addr + offset, pte)?;
        }

        Ok(())
    }

    /// This is a recursive function used to traverse the page table hierarchy for a given virtual
    /// address range and the given physical address of the root page table of the page table
    /// hierarchy. It invokes the appropriate user callbacks in [`crate::walker::PageWalker`],
    /// while traversing the page tables.
    pub fn walk_mut<PageWalkerMut, Mapper, Error>(
        &self,
        phys_addr: u64,
        range: Range<usize>,
        walker: &mut PageWalkerMut,
        mapper: &mut Mapper,
    ) -> Result<(), Error>
    where
        PageWalkerMut: crate::walker::PageWalkerMut<Mapper, Error>,
        Mapper: crate::address_space::PageTableMapper<Error>,
    {
        self.do_walk_mut(phys_addr, self.levels.len() - 1, range, walker, mapper)
    }
}
