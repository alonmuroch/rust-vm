pub struct Config;

impl Config {
    pub const MAX_INPUT_LEN: usize = 1024;
    pub const CODE_SIZE_LIMIT: usize = 0x8000;  // Increased from 12KB to 32KB to support external libraries
    pub const RO_DATA_SIZE_LIMIT: usize = 0x2000;  // Increased from 4KB to 8KB
    pub const HEAP_START_ADDR: usize = Self::CODE_SIZE_LIMIT + Self::RO_DATA_SIZE_LIMIT + 0x100;
    pub const MAX_RESULT_SIZE: usize = types::result::RESULT_SIZE;

    pub const PROGRAM_START_ADDR: u32 = 0x400;
    pub const RESULT_ADDR: u32 = 0x100;
}
