//! This module provides the [`AddressSpace`] struct which provides an abstraction over a virtual
//! address space and provides methods to introspect and manage the virtual address space.

use core::marker::PhantomData;
use core::ops::Range;
use crate::{PageFormat, PteType};
use num_traits::{PrimInt, Unsigned};

/// The [`AddressSpace`] struct expects a type implementing this trait in order to map the page
/// tables while performing the various page table operations.
pub trait PageTableMapper<PTE, PageTable, PageTableMut, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
{
    /// An `Error` constant indicating that the PTE was not found.
    const PTE_NOT_FOUND: Error;

    /// Given the physical address, maps in the physical page backing the page table. To unmap the
    /// page upon use, the type implementing [`crate::table::PageTable`] must implement
    /// [`core::ops::Drop`] semantics.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTable, Error>;

    /// Given the physical address, maps in the physical page backing the page table. To unmap the
    /// page upon use, the type implementing [`crate::table::PageTableMut`] must implement
    /// [`core::ops::Drop`] semantics.
    fn map_table_mut(&self, phys_addr: PTE) -> Result<PageTableMut, Error>;
}

/// The [`PteReader`] struct is an implementation of a [`crate::walker::PageWalker`] used to
/// retrieve the PTE for a given virtual address, which is used by the [`AddressSpace::read_pte`]
/// method.
struct PteReader<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    mapper: &'a Mapper,
    /// Storage for the retrieved PTE.
    pte: Option<PTE>,
    /// A marker for PageTable.
    page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalker<PTE, PageTable, Error> for PteReader<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Uses the page table mapper to map the page table backing the physical address.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTable, Error> {
        self.mapper.map_table(phys_addr)
    }

    /// Stores the PTE of the page, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &PTE) -> Result<(), Error> {
        if pte_type.is_page() {
            self.pte = Some(*pte);
        }

        Ok(())
    }
}

/// The [`PteWriter`] struct is an implementation of a [`crate::walker::PageWalkerMut`] used to
/// store the PTE for a given virtual address, which is used by the [`AddressSpace::write_pte`]
/// method.
struct PteWriter<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table mapper.
    mapper: &'a Mapper,
    /// The PTE to store.
    pte: PTE,
    /// A marker for PageTable.
    page_table: PhantomData<PageTable>,
    /// A marker for PageTableMut.
    page_table_mut: PhantomData<PageTableMut>,
    /// A marker for Error.
    error: PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> crate::PageWalkerMut<PTE, PageTableMut, Error> for PteWriter<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Uses the page table mapper to map the page table backing the physical address.
    fn map_table(&self, phys_addr: PTE) -> Result<PageTableMut, Error> {
        self.mapper.map_table_mut(phys_addr)
    }

    /// Store the PTE, if the virtual address resolves to a page.
    fn handle_pte(&mut self, pte_type: PteType, _range: Range<usize>, pte: &mut PTE) -> Result<(), Error> {
        if let PteType::Page(_) = pte_type {
            *pte = self.pte;
        }

        Ok(())
    }
}

/// Abstracts a virtual address space.
pub struct AddressSpace<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// The page table format describing the page table hierarchy for this virtual address space.
    format: PageFormat<'a, PTE>,

    /// The root address of the page table hierarchy.
    root: PTE,

    /// The type implementing PageTableMapper.
    mapper: Mapper,

    /// A marker for PageTable.
    page_table: core::marker::PhantomData<PageTable>,

    /// A marker for PageTableMut.
    page_table_mut: core::marker::PhantomData<PageTableMut>,

    /// A marker for Error.
    error: core::marker::PhantomData<Error>,
}

impl<'a, PTE, PageTable, PageTableMut, Mapper, Error> AddressSpace<'a, PTE, PageTable, PageTableMut, Mapper, Error>
where
    PTE: PrimInt + Unsigned,
    PageTable: crate::PageTable<PTE>,
    PageTableMut: crate::PageTableMut<PTE>,
    Mapper: PageTableMapper<PTE, PageTable, PageTableMut, Error>,
{
    /// Creates a new address space for the given page table format descripting the page table
    /// hierarchy, the page table mapper and the pointer to the root of the page table
    /// hierarchy.
    pub fn new(format: PageFormat<'a, PTE>, mapper: Mapper, root: PTE) -> Self {
        Self {
            format,
            mapper,
            root,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        }
    }

    /// Reads the PTE for the given the virtual address if the virtual address is valid.
    pub fn read_pte(&self, virt_addr: usize) -> Result<PTE, Error> {
        let mut reader = PteReader {
            mapper: &self.mapper,
            pte: None,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk(self.root, virt_addr..virt_addr + 1, &mut reader)?;

        match reader.pte {
            Some(pte) => Ok(pte),
            _ => Err(Mapper::PTE_NOT_FOUND),
        }
    }

    /// Writes the PTE for the given virtual address if the virtual address is valid.
    pub fn write_pte(&self, virt_addr: usize, pte: PTE) -> Result<(), Error> {
        let mut writer = PteWriter {
            mapper: &self.mapper,
            pte,
            page_table: PhantomData,
            page_table_mut: PhantomData,
            error: PhantomData,
        };

        self.format.walk_mut(self.root, virt_addr..virt_addr + 1, &mut writer)?;

        Ok(())
    }
}
