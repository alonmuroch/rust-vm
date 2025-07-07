pub struct Config;

impl Config {
    pub const MAX_INPUT_LEN: usize = 1024;
    pub const CODE_SIZE_LIMIT: usize = 0x3000;
    pub const RO_DATA_SIZE_LIMIT: usize = 0x1000;
    pub const HEAP_START_ADDR: usize = Self::CODE_SIZE_LIMIT + Self::RO_DATA_SIZE_LIMIT + 0x100;

    pub const PROGRAM_START_ADDR: u32 = 0x100;
}
