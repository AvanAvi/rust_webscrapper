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

impl Row {
    /// Walks names with badly named loop, closure, and match bindings.
    pub fn walk(&self, vec_str_names: &[String], opt_u_limit: Option<u32>) -> usize {
        for name in vec_str_names {
            println!("{name}");
        }
        let u_short = vec_str_names.iter().filter(|n| n.len() < 3).count();
        let u_sizes: usize = vec_str_names.iter().map(|size: &String| size.len()).sum();
        let u_flags = vec_str_names.iter().filter(|b_flag: &&String| b_flag.is_empty()).count();
        if let Some(limit) = opt_u_limit {
            return limit as usize;
        }
        match opt_u_limit {
            Some(found) if vec_str_names.iter().any(|x| x.len() == found as usize) => 1,
            None => u_short + u_sizes + u_flags,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn checks_total() {}
}
