use state::State;
use super::utils::to_address;

/// Build a test state with prefunded accounts.
pub fn test_state() -> State {
    let mut state = State::new();

    // Prefund the addresses used in examples tests.
    for addr_hex in [
        "d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d2",
        "d5a3c7f85d2b6e91fa78cd3210b45f6ae913d0d3",
    ] {
        let addr = to_address(addr_hex);
        let account = state.get_account_mut(&addr);
        account.balance = 1_000_000_000u128; // 1 billion am for testing
    }

    state
}
