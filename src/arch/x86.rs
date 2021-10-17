//! This module provides the page table formats available for the x86 architecture.
use lazy_static::lazy_static;
use crate::{PageFormat, PageLevel};

lazy_static! {
    /// A page table layout for x86 consisting of two page levels with 32-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 1024 entries and uses 10 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 4M huge pages.
    pub static ref PAGE_FORMAT_4K: PageFormat<'static, u32> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 10,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
            },
            PageLevel {
                shift_bits: 22,
                va_bits: 10,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 7, 1 << 7),
            },
        ],
        physical_mask: 0xffff_f000,
    };

    /// A page table layout for x86 consisting of three page levels with 64-bit PTEs, through
    /// the Physical Address Extension (PAE) feature, and a page size of 4K. Therefore, each page
    /// table has 512 entries and uses 9 bits of the virtual address to index into the page table,
    /// except for the root page table. The root page table has four entries and uses 2 bits of the
    /// virtual address to index into the page table. Furthermore, it supports 2M huge pages.
    pub static ref PAGE_FORMAT_4K_PAE: PageFormat<'static, u64> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 9,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
            },
            PageLevel {
                shift_bits: 21,
                va_bits: 9,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (1 << 7, 1 << 7),
            },
            PageLevel {
                shift_bits: 30,
                va_bits: 2,
                present_bit: (1 << 0, 1 << 0),
                huge_page_bit: (0, 0),
            },
        ],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// The default page format is a two-level page table hierarchy with 4K pages.
    pub static ref DEFAULT_PAGE_FORMAT: PageFormat<'static, u32> = PAGE_FORMAT_4K.clone();
}
