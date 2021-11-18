//! This module provides the page table formats available for the x86 architecture.
use lazy_static::lazy_static;
use crate::{PageFormat, PageLevel};

/// The page is present.
pub const PAGE_PRESENT: u64 = 1 << 0;
/// The page is writeable.
pub const PAGE_WRITE:   u64 = 1 << 1;
/// The page is accessible in user mode.
pub const PAGE_USER:    u64 = 1 << 2;
/// The page is a huge page.
pub const PAGE_HUGE:    u64 = 1 << 7;

lazy_static! {
    /// A page table layout for x86 consisting of two page levels with 32-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 1024 entries and uses 10 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 4M huge pages.
    pub static ref PAGE_FORMAT_4K: PageFormat<'static> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 10,
                present_bit: (PAGE_PRESENT, PAGE_PRESENT),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 22,
                va_bits: 10,
                present_bit: (PAGE_PRESENT, PAGE_PRESENT),
                huge_page_bit: (PAGE_HUGE, PAGE_HUGE),
                page_table_mask: PAGE_PRESENT | PAGE_WRITE | PAGE_USER,
            },
        ],
        physical_mask: 0xffff_f000,
    };

    /// A page table layout for x86 consisting of three page levels with 64-bit PTEs, through
    /// the Physical Address Extension (PAE) feature, and a page size of 4K. Therefore, each page
    /// table has 512 entries and uses 9 bits of the virtual address to index into the page table,
    /// except for the root page table. The root page table has four entries and uses 2 bits of the
    /// virtual address to index into the page table. Furthermore, it supports 2M huge pages.
    pub static ref PAGE_FORMAT_4K_PAE: PageFormat<'static> = PageFormat {
        levels: &[
            PageLevel {
                shift_bits: 12,
                va_bits: 9,
                present_bit: (PAGE_PRESENT as u64, PAGE_PRESENT as u64),
                huge_page_bit: (0, 0),
                page_table_mask: 0,
            },
            PageLevel {
                shift_bits: 21,
                va_bits: 9,
                present_bit: (PAGE_PRESENT as u64, PAGE_PRESENT as u64),
                huge_page_bit: (PAGE_HUGE as u64, PAGE_HUGE as u64),
                page_table_mask: (PAGE_PRESENT | PAGE_WRITE | PAGE_USER) as u64,
            },
            PageLevel {
                shift_bits: 30,
                va_bits: 2,
                present_bit: (PAGE_PRESENT as u64, PAGE_PRESENT as u64),
                huge_page_bit: (0, 0),
                page_table_mask: (PAGE_PRESENT | PAGE_WRITE | PAGE_USER) as u64,
            },
        ],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// The default page format is a two-level page table hierarchy with 4K pages.
    pub static ref DEFAULT_PAGE_FORMAT: PageFormat<'static> = PAGE_FORMAT_4K.clone();
}
