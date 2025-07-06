/// Magic number identifying an ELF file (`0x7F + "ELF"` in little endian).
pub const ELF_MAGIC: u32 = 0x464C457F;

/// ELF file header.
///
/// Appears at the beginning of every ELF binary.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ElfHeader {
    /// Magic number (must be `ELF_MAGIC`)
    pub magic: u32,
    /// ELF identification bytes (includes class, endian, version, OS ABI, etc.)
    pub ident: [u8; 12],
    /// Object file type (e.g., 2 = executable)
    pub file_type: u16,
    /// Target machine architecture (e.g., 3 = x86)
    pub machine: u16,
    /// ELF version (always 1)
    pub version: u32,
    /// Virtual address of the entry point
    pub entry: u32,
    /// Offset in bytes to the program header table
    pub program_header_offset: u32,
    /// Offset in bytes to the section header table
    pub section_header_offset: u32,
    /// Processor-specific flags (usually 0)
    pub flags: u32,
    /// Size of this header (in bytes)
    pub header_size: u16,
    /// Size of each entry in the program header table
    pub program_header_entry_size: u16,
    /// Number of entries in the program header table
    pub program_header_count: u16,
    /// Size of each entry in the section header table
    pub section_header_entry_size: u16,
    /// Number of entries in the section header table
    pub section_header_count: u16,
    /// Index of the section name string table
    pub section_name_string_index: u16,
}
const _: () = assert!(core::mem::size_of::<ElfHeader>() == 52);

/// ELF Program Header.
///
/// Describes a single segment to be loaded into memory.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    /// Segment type (e.g., 1 = LOAD)
    pub segment_type: u32,
    /// Offset of segment in file
    pub offset: u32,
    /// Virtual address to load the segment
    pub virtual_address: u32,
    /// Physical address (used by OS/bootloader)
    pub physical_address: u32,
    /// Size of segment in the file
    pub file_size: u32,
    /// Size of segment in memory (may be larger than file_size)
    pub memory_size: u32,
    /// Segment flags (e.g., readable, writable, executable)
    pub flags: u32,
    /// Required alignment of segment in memory and file
    pub alignment: u32,
}
const _: () = assert!(core::mem::size_of::<ProgramHeader>() == 32);

/// Segment type for program headers.
pub const ELF_PROG_LOAD: u32 = 1;

/// Segment permission flags.
pub const ELF_PROG_FLAG_EXEC: u32 = 1;
pub const ELF_PROG_FLAG_WRITE: u32 = 2;
pub const ELF_PROG_FLAG_READ: u32 = 4;
