#[derive(Copy, Clone, Debug)]
pub struct Result {
    pub success: bool,
    pub error_code: u32,
}