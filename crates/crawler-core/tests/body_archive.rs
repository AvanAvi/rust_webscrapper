//! In-memory body archive: per-body cap, byte capacity, entry limit, deduplication, and
//! eviction of unreferenced bodies only.

use crawler_core::{BodyArchive, CoreError, MemoryBodyArchive, U_MAX_ARCHIVED_BODY_BYTES};

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_body_over_the_per_body_cap_is_refused() {
    let o_archive = MemoryBodyArchive::new();
    let res_str = o_archive.store(&vec![b'a'; U_MAX_ARCHIVED_BODY_BYTES + 1]);
    assert!(matches!(
        res_str,
        Err(CoreError::ArchiveBodyTooLarge { .. })
    ));
    assert!(
        o_archive
            .store(&vec![b'a'; U_MAX_ARCHIVED_BODY_BYTES])
            .is_ok()
    );
}

#[test]
fn test_same_body_is_stored_once_and_kept_while_referenced() -> TestResult {
    let o_archive = MemoryBodyArchive::with_limits(16, 4);
    let str_sha = o_archive.store(b"0123456789")?;
    assert_eq!(o_archive.store(b"0123456789")?, str_sha);
    assert_eq!(o_archive.usage(), (10, 1));
    o_archive.release(&str_sha);
    assert!(
        o_archive.store(b"abcdefghij").is_err(),
        "one reference still holds the body"
    );
    o_archive.release(&str_sha);
    o_archive.store(b"abcdefghij")?;
    assert_eq!(o_archive.load(&str_sha), None, "evicted once unreferenced");
    Ok(())
}

#[test]
fn test_byte_capacity_fails_when_every_body_is_referenced() -> TestResult {
    let o_archive = MemoryBodyArchive::with_limits(16, 100);
    o_archive.store(b"0123456789")?;
    let res_str = o_archive.store(b"abcdefghij");
    assert!(matches!(
        res_str,
        Err(CoreError::ArchiveFull { u_capacity: 16 })
    ));
    assert_eq!(o_archive.usage(), (10, 1));
    Ok(())
}

#[test]
fn test_entry_limit_evicts_oldest_unreferenced_then_fails() -> TestResult {
    let o_archive = MemoryBodyArchive::with_limits(1024, 2);
    let str_first = o_archive.store(b"a")?;
    let str_second = o_archive.store(b"b")?;
    o_archive.release(&str_first);
    o_archive.store(b"c")?;
    assert_eq!(o_archive.load(&str_first), None);
    assert!(o_archive.load(&str_second).is_some());
    let res_str = o_archive.store(b"d");
    assert!(matches!(
        res_str,
        Err(CoreError::ArchiveTooManyEntries { u_max_entries: 2 })
    ));
    assert_eq!(o_archive.usage(), (2, 2));
    Ok(())
}
