#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Result {
    pub success: bool,
    pub error_code: u32,
}