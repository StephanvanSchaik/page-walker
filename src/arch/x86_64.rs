//! This module provides the page table formats available for the x86-64 architecture.
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


static PAGE_LEVELS_4K: &'static [PageLevel<u64>] = &[
    PageLevel {
        shift_bits: 12,
        va_bits: 9,
        present_bit: (PAGE_PRESENT, PAGE_PRESENT),
        huge_page_bit: (0, 0),
        page_table_mask: 0,
    },
    PageLevel {
        shift_bits: 21,
        va_bits: 9,
        present_bit: (PAGE_PRESENT, PAGE_PRESENT),
        huge_page_bit: (PAGE_HUGE, PAGE_HUGE),
        page_table_mask: PAGE_PRESENT | PAGE_WRITE | PAGE_USER,
    },
    PageLevel {
        shift_bits: 30,
        va_bits: 9,
        present_bit: (PAGE_PRESENT, PAGE_PRESENT),
        huge_page_bit: (PAGE_HUGE, PAGE_HUGE),
        page_table_mask: PAGE_PRESENT | PAGE_WRITE | PAGE_USER,
    },
    PageLevel {
        shift_bits: 39,
        va_bits: 9,
        present_bit: (PAGE_PRESENT, PAGE_PRESENT),
        huge_page_bit: (0, 0),
        page_table_mask: PAGE_PRESENT | PAGE_WRITE | PAGE_USER,
    },
    PageLevel {
        shift_bits: 48,
        va_bits: 9,
        present_bit: (PAGE_PRESENT, PAGE_PRESENT),
        huge_page_bit: (0, 0),
        page_table_mask: PAGE_PRESENT | PAGE_WRITE | PAGE_USER,
    },
];

lazy_static! {
    /// A page table layout for x86-64 consisting of four page levels with 64-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 512 entries and uses 9 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 2M huge page and optionally
    /// 1G huge pages. Finally, while the number of physical address bits supported is
    /// CPU-specific, the maximum is 52 bits.
    pub static ref PAGE_FORMAT_4K_L4: PageFormat<'static, u64> = PageFormat {
        levels: &PAGE_LEVELS_4K[0..4],
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// A page table layout for x86-64 consisting of five page levels with 64-bit PTEs and a page
    /// size of 4K. Therefore, each page table has 512 entries and uses 9 bits of the virtual
    /// address to index into the page table. Furthermore, it supports 2M huge page and optionally
    /// 1G huge pages. Finally, while the number of physical address bits supported is
    /// CPU-specific, the maximum is 52 bits.
    pub static ref PAGE_FORMAT_4K_L5: PageFormat<'static, u64> = PageFormat {
        levels: PAGE_LEVELS_4K,
        physical_mask: 0x000f_ffff_ffff_f000,
    };

    /// The five-level page table layout is also known as LA57 as it expands linear or virtual
    /// addresses to 57 bits.
    pub static ref PAGE_FORMAT_LA57: PageFormat<'static, u64> = PAGE_FORMAT_4K_L5.clone();

    /// The default page format is a four-level page table hierarchy with 4K pages.
    pub static ref DEFAULT_PAGE_FORMAT: PageFormat<'static, u64> = PAGE_FORMAT_4K_L4.clone();
}
