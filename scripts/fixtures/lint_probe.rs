//! Lint probe: each item below must trip exactly one configured lint.
//! Not part of the workspace; copied into a scratch crate by scripts/lint-probe.sh.

/// Probe type for the receiver-counting argument check.
pub struct Probe;

impl Probe {
    /// Trips too_many_arguments: five parameters plus `self` is 6 > 5.
    pub fn too_many(&self, u_a: u8, u_b: u8, u_c: u8, u_d: u8, u_e: u8) -> u8 {
        u_a + u_b + u_c + u_d + u_e
    }
}

/// Trips cognitive_complexity: clippy scores this 9 > 8.
pub fn complex(u_a: u32, u_b: u32) -> u32 {
    let mut u_total = 0;
    for u_i in 0..u_a {
        if u_i % 2 == 0 && u_b > 1 || u_b == 7 {
            if u_i % 3 == 0 || u_b > 5 && u_a > 2 {
                u_total += 1;
            } else if u_i % 5 == 0 {
                while u_total < 10 {
                    if u_b == 3 { u_total += 2; } else { u_total += 1; }
                }
            }
        }
    }
    if u_total > 3 { u_total -= 1; }
    if u_total > 4 && u_a > 1 { u_total -= 1; }
    u_total
}

/// Trips too_many_lines: 30 body lines > 25.
pub fn long() -> u32 {
    let mut u_x = 0;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x += 1;
    u_x
}

/// Trips unwrap_used.
pub fn unwraps(opt_u: Option<u8>) -> u8 {
    opt_u.unwrap()
}

/// Trips expect_used.
pub fn expects(opt_u: Option<u8>) -> u8 {
    opt_u.expect("present")
}

/// Trips panic.
pub fn panics() {
    panic!("probe");
}

/// Trips todo.
pub fn todos() {
    todo!();
}

/// Trips unsafe_code.
pub fn unsafes() -> u8 {
    unsafe { std::ptr::read(&1u8) }
}

pub fn undocumented() {}
