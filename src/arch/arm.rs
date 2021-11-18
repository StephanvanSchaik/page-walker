//! This module provides the page table formats available for the ARMv7-A architecture.
use lazy_static::lazy_static;
use crate::{PageFormat, PageLevel};

lazy_static! {
    /// A page table layout for ARMv7-A consisting of two page levels with 32-bit PTEs and a page
    /// size of 4K. The leaf page table has 256 entries and uses 8 bits of the virtual address to
    /// index into the page table, whereas the root page table has 4096 entries and uses 12 bits of
    /// the virtual address to index into the page table. Furthermore, it supports 1M huge pages.
    pub static ref PAGE_FORMAT_4K: PageFormat<'static> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 8,
                present_bit: (1 << 0 | 1 << 1, 1 << 0 | 1 << 1),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 20,
                va_bits: 12,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 1, 0),
                page_table_mask: 0,
            },
        ],
        physical_mask: 0xffff_f000,
        pte_size: core::mem::size_of::<u64>(),
    };

    /// A page table layout for ARMv7-A consisting of three page levels with 64-bit PTEs, through
    /// the Long Physical Address Extension (LPAE) feature, and a page size of 4K. Therefore, each
    /// page table has 512 entries and uses 9 bits of the virtual address to index into the page
    /// table, except for the root page table. The root page table has four entries and uses 2 bits
    /// of the virtual address to index into the page table. Furthermore, it supports 2M huge
    /// pages and 1G huge pages.
    pub static ref PAGE_FORMAT_4K_PAE: PageFormat<'static> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 9,
                present_bit: (1 << 0 | 1 << 1, 1 << 0 | 1 << 1),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 21,
                va_bits: 9,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 1, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 30,
                va_bits: 2,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 1, 0),
                page_table_mask: 0,
            },
        ],
        physical_mask: 0x0000_00ff_ffff_f000,
        pte_size: core::mem::size_of::<u64>(),
    };

    /// The default page format is a two-level page table hierarchy with 4K pages.
    pub static ref DEFAULT_PAGE_FORMAT: PageFormat<'static> = PAGE_FORMAT_4K.clone();
}
