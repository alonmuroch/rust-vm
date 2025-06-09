#[derive(Copy, Clone)]
pub enum Register {
    Zero = 0, // x0: hardwired zero
    Ra = 1,   // x1: return address
    Sp = 2,   // x2: stack pointer
    Gp = 3,   // x3: global pointer
    Tp = 4,   // x4: thread pointer

    T0 = 5, // x5: temporary register
    T1 = 6, // x6: temporary register
    T2 = 7, // x7: temporary register

    S0 = 8, // x8: saved register / frame pointer
    S1 = 9, // x9: saved register

    A0 = 10, // x10: argument 0 / return value
    A1 = 11, // x11: argument 1 / return value (if needed)
    A2 = 12, // x12: argument 2
    A3 = 13, // x13: argument 3
    A4 = 14, // x14: argument 4
    A5 = 15, // x15: argument 5
    A6 = 16, // x16: argument 6
    A7 = 17, // x17: argument 7

    S2 = 18,  // x18: saved register
    S3 = 19,  // x19: saved register
    S4 = 20,  // x20: saved register
    S5 = 21,  // x21: saved register
    S6 = 22,  // x22: saved register
    S7 = 23,  // x23: saved register
    S8 = 24,  // x24: saved register
    S9 = 25,  // x25: saved register
    S10 = 26, // x26: saved register
    S11 = 27, // x27: saved register

    T3 = 28, // x28: temporary register
    T4 = 29, // x29: temporary register
    T5 = 30, // x30: temporary register
    T6 = 31, // x31: temporary register
}
