//! This module provides the page table formats available for the AArch64 architecture.
use lazy_static::lazy_static;
use crate::{PageFormat, PageLevel};

static PAGE_LEVELS_4K: &'static [PageLevel<u64>] = &[
    PageLevel {
        shift_bits: 12,
        va_bits: 9,
        present_bit: (1 << 0, 1 << 0),
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
        va_bits: 9,
        present_bit: (1 << 0, 1 << 0),
        huge_page_bit: (1 << 1, 0),
        page_table_mask: 0,
    },
    PageLevel {
        shift_bits: 39,
        va_bits: 9,
        present_bit: (1 << 0, 1 << 0),
        huge_page_bit: (0, 0),
        page_table_mask: 0,
    },
];

lazy_static! {
    /// A page table layout for AArch64 consisting of three page levels with 64-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 512 entries and uses 9 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 2M huge pages and 1G huge
    /// pages. Finally, while the number of physical address bits supported is CPU-specific, the
    /// maximum is 52 bits. This format is commonly used instead of `PAGE_FORMAT_4K_L4` to reduce
    /// the depth of the page table walk to improve the performance of virtual address translation.
    pub static ref PAGE_FORMAT_4K_L3: PageFormat<'static, u64> = PageFormat {
        levels: &PAGE_LEVELS_4K[0..3],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// A page table layout for AArch64 consisting of four page levels with 64-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 512 entries and uses 9 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 2M huge pages and 1G huge
    /// pages. Finally, while the number of physical address bits supported is CPU-specific, the
    /// maximum is 52 bits.
    pub static ref PAGE_FORMAT_4K_L4: PageFormat<'static, u64> = PageFormat {
        levels: &PAGE_LEVELS_4K[0..3],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// A page table layout for AArch64 consisting of four page levels with 64-bit PTEs and a page
    /// size of 16K. Therefore, each page table has 2048 entries and uses 11 bits of the virtual
    /// address to index into the page table, except for the root page table. The root page table
    /// instead only consists of two entries and only uses 1 bit of the virtual address to index
    /// into this page table. Finally, while the number of physical address bits supported is
    /// CPU-specific, the maximum is 52 bits. This page table format is rather exotic.
    pub static ref PAGE_FORMAT_16K: PageFormat<'static, u64> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 11,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 23,
                va_bits: 11,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 1, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 34,
                va_bits: 11,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 45,
                va_bits: 1,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
        ],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// A page table layout for AArch64 consisting of three page levels with 64-bit PTEs and a page
    /// size of 64K. Therefore, each page table has 8192 entries and uses 13 bits of the virtual
    /// address to index into the page table, except for the root page table. The root page table
    /// instead only consists of 64 entries and only uses 6 bit of the virtual address to index
    /// into this page table. Finally, while the number of physical address bits supported is
    /// CPU-specific, the maximum is 52 bits. This page table format is rather exotic.
    pub static ref PAGE_FORMAT_64K: PageFormat<'static, u64> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 13,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 25,
                va_bits: 13,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 1, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 38,
                va_bits: 6,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
        ],
        physical_mask: 0x000f_ffff_ffff_f000,
    };
}
