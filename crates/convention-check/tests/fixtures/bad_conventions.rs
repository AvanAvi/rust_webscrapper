//! Fixture that breaks every rule exactly as counted in `check_fixtures.rs`.

const max_items: u32 = 3;

/// Constant whose prefix does not match its type.
const STR_LIMIT: u32 = 3;

fn helper() {}

/// Struct whose fields break naming, doc, and rename rules.
#[derive(Serialize)]
pub struct Report {
    /// Field without a prefix.
    count: u32,
    /// Field with the wrong prefix.
    str_total: u32,
    u_undocumented: u32,
}

/// Database row without sqlx renames.
#[derive(FromRow)]
pub struct Row {
    /// Signed id with an unsigned prefix.
    u_id: i64,
}

pub struct Undocumented;

impl Report {
    pub fn total(&self, value: u32) -> u32 {
        let doubled = value * 2;
        let str_count: u32 = doubled;
        str_count
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn checks_total() {}
}
