use avm::memory::MemoryPage;
use vm::metering::{MemoryAccessKind, NoopMeter};

#[test]
fn test_offset_zero_base() {
    let mem = MemoryPage::new_with_base(1024, 0);
    let mut meter = NoopMeter::default();
    assert_eq!(mem.offset(0), 0);
    assert_eq!(mem.offset(100), 100);
    assert_eq!(mem.offset(1023), 1023);
    assert_eq!(mem.load_u32(0, &mut meter, MemoryAccessKind::Load), Some(0));
}

#[test]
fn test_offset_high_base() {
    let base = 0x80000000;
    let mem = MemoryPage::new_with_base(1024, base);
    assert_eq!(mem.offset(base), 0);
    assert_eq!(mem.offset(base + 100), 100);
    assert_eq!(mem.offset(base + 1023), 1023);
}

#[test]
#[should_panic(expected = "Address below base_address")]
fn test_offset_below_base_panics() {
    let base = 0x80000000;
    let mem = MemoryPage::new_with_base(1024, base);
    mem.offset(base - 1);
}

#[test]
fn test_store_and_load_zero_base() {
    let mem = MemoryPage::new_with_base(1024, 0);
    let mut meter = NoopMeter::default();
    assert!(mem.store_u8(10, 0xAB, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_byte(10, &mut meter, MemoryAccessKind::Load),
        Some(0xAB)
    );
    assert!(mem.store_u16(20, 0xCDEF, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_halfword(20, &mut meter, MemoryAccessKind::Load),
        Some(0xCDEF)
    );
    assert!(mem.store_u32(30, 0x12345678, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_u32(30, &mut meter, MemoryAccessKind::Load),
        Some(0x12345678)
    );
}

#[test]
fn test_store_and_load_high_base() {
    let base = 0x80000000;
    let mem = MemoryPage::new_with_base(1024, base);
    let mut meter = NoopMeter::default();
    assert!(mem.store_u8(base + 10, 0xAB, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_byte(base + 10, &mut meter, MemoryAccessKind::Load),
        Some(0xAB)
    );
    assert!(mem.store_u16(base + 20, 0xCDEF, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_halfword(base + 20, &mut meter, MemoryAccessKind::Load),
        Some(0xCDEF)
    );
    assert!(mem.store_u32(base + 30, 0x12345678, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_u32(base + 30, &mut meter, MemoryAccessKind::Load),
        Some(0x12345678)
    );
}

#[test]
fn test_store_and_load_at_offset_zero() {
    let base = 0x80000000;
    let mem = MemoryPage::new_with_base(1024, base);
    let mut meter = NoopMeter::default();
    assert!(mem.store_u8(base, 0xAA, &mut meter, MemoryAccessKind::Store));
    assert_eq!(
        mem.load_byte(base, &mut meter, MemoryAccessKind::Load),
        Some(0xAA)
    );
}
