//! This module provides the [`PageLevel`] struct used to describe a single level in a page table
//! hierarchy. The full page table hierarchy is described by [`crate::format::PageFormat`].

use num_traits::{PrimInt, Unsigned};

/// Describes a single page level of the page hierarchy.
#[derive(Clone, Debug)]
pub struct PageLevel<PTE>
where
    PTE: PrimInt + Unsigned,
{
    /// The number of bits to shift right in the virtual address to get the index bits for this
    /// page level.
    pub shift_bits: usize,
    /// The number of bits in the virtual address to extract as the index for this page level.
    pub va_bits: usize,
    /// The present bit in the PTE. The first mask is to select the relevants bits, the second is
    /// what the value should be upon masking.
    pub present_bit: (PTE, PTE),
    /// The huge page bit in the PTE. If the current page level does not support huge pages, then
    /// this should be set to zero. The first mask is to select the relevant bits, the second is
    /// what the value should be upon masking.
    pub huge_page_bit: (PTE, PTE),
}

impl<PTE> PageLevel<PTE>
where
    PTE: PrimInt + Unsigned,
{
    /// Calculates the number of entries present in a page table for this page level.
    pub fn entries(&self) -> usize {
        1 << self.va_bits
    }

    /// Calculates the page size for this page level.
    pub fn page_size(&self) -> usize {
        1 << self.shift_bits
    }

    /// Calculates the shifted mask to select the appropriate bits from the virtual address.
    pub fn mask(&self) -> usize {
        ((1 << self.va_bits) - 1) << self.shift_bits
    }

    /// Calculates the last virtual address within the same page for the current page level of the
    /// given a virtual address. This can be used to retrieve the first address of the next page
    /// for the current page level by simply adding one.
    pub fn end(&self, addr: usize) -> usize {
        addr | (self.page_size() - 1)
    }

    /// Calculates the PTE index of the given virtual address for the current page table level,
    /// which can then be used to index into the page table to get the corresponding PTE for this
    /// virtual address.
    pub fn pte_index(&self, addr: usize) -> usize {
        (addr >> self.shift_bits) & ((1 << self.va_bits) - 1)
    }

    /// Given a PTE, it checks if the PTE points to a present page or page table.
    pub fn is_present(&self, pte: PTE) -> bool {
        (pte & self.present_bit.0) == self.present_bit.1
    }

    /// Given a PTE, it checks if the PTE points to a huge page. Always returns `false` if the
    /// current page level does not support huge pages.
    pub fn is_huge_page(&self, pte: PTE) -> bool {
        if self.huge_page_bit.0 != PTE::zero() {
            let mask = self.present_bit.0 | self.huge_page_bit.0;
            let value = self.present_bit.1 | self.huge_page_bit.1;

            (pte & mask) == value
        } else {
            false
        }
    }

    /// Returns the bits to set for a PTE referring to a page table.
    pub fn page_table_mask(&self) -> PTE {
        PTE::zero()
    }
}
