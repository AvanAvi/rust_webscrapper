//! Fetches, caches, archives, and parses per-host signal files, and reads page-level signals.

pub mod backoff_policy;
pub mod collector_config;
pub mod fetched_file;
pub mod host_cache;
pub mod leader_guard;
pub mod page_signal_reader;
pub mod signal_collector;
pub mod signal_document;
pub mod signal_file;
pub mod signal_file_fetcher;
