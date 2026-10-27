//! Production address policy: every non-public range is refused, including embedded IPv4 forms.

use std::net::IpAddr;
use std::sync::Arc;

use crawler_fetch::{AddressPolicy, FetchError, ProductionAddressPolicy, UrlSafety};
use url::Url;

/// Result type for tests that use `?` instead of `expect`.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Addresses that must be refused.
const VEC_STR_BLOCKED: &[&str] = &[
    "127.0.0.1",
    "127.255.255.254",
    "10.1.2.3",
    "172.16.0.1",
    "172.31.255.255",
    "192.168.1.1",
    "169.254.169.254",
    "169.254.0.1",
    "100.64.0.1",
    "100.127.255.255",
    "100.100.100.200",
    "0.0.0.0",
    "0.1.2.3",
    "224.0.0.1",
    "239.255.255.250",
    "255.255.255.255",
    "240.0.0.1",
    "192.0.2.1",
    "198.51.100.7",
    "203.0.113.9",
    "198.18.0.1",
    "192.0.0.192",
    "::",
    "::1",
    "fe80::1",
    "fc00::1",
    "fd00:ec2::254",
    "ff02::1",
    "2001:db8::1",
    "3fff::1",
    "100::1",
    "fec0::1",
    "2001::1",
    "::ffff:127.0.0.1",
    "::ffff:10.0.0.1",
    "::ffff:169.254.169.254",
    "::ffff:0.0.0.0",
    "::127.0.0.1",
    "::8.8.8.8",
    "64:ff9b::a00:1",
    "64:ff9b:1::1",
    "2002:a00:1::",
    "2002:7f00:1::",
];

/// Addresses that must be allowed.
const VEC_STR_ALLOWED: &[&str] = &[
    "8.8.8.8",
    "1.1.1.1",
    "93.184.215.14",
    "172.32.0.1",
    "100.128.0.1",
    "11.0.0.1",
    "169.255.0.1",
    "2606:4700:4700::1111",
    "2001:4860:4860::8888",
    "::ffff:8.8.8.8",
    "64:ff9b::808:808",
    "2002:808:808::",
];

/// Odd URL spellings of loopback/metadata and what the url crate normalizes the host to.
const VEC_O_ODD_LITERALS: &[(&str, &str)] = &[
    ("http://0x7f.1/", "127.0.0.1"),
    ("http://2130706433/", "127.0.0.1"),
    ("http://0177.0.0.1/", "127.0.0.1"),
    ("http://127.1/", "127.0.0.1"),
    ("http://0x7f000001/", "127.0.0.1"),
    ("http://0251.0376.0251.0376/", "169.254.169.254"),
    ("http://2852039166/", "169.254.169.254"),
    ("http://[::ffff:127.0.0.1]/", "[::ffff:7f00:1]"),
    ("http://[0:0:0:0:0:ffff:a9fe:a9fe]/", "[::ffff:a9fe:a9fe]"),
];

#[test]
fn test_blocked_ranges_are_refused() -> TestResult {
    let o_policy = ProductionAddressPolicy::new();
    for str_ip in VEC_STR_BLOCKED {
        assert!(
            o_policy.check(str_ip.parse::<IpAddr>()?).is_err(),
            "{str_ip} must be blocked"
        );
    }
    Ok(())
}

#[test]
fn test_public_addresses_are_allowed() -> TestResult {
    let o_policy = ProductionAddressPolicy::new();
    for str_ip in VEC_STR_ALLOWED {
        assert_eq!(
            o_policy.check(str_ip.parse::<IpAddr>()?),
            Ok(()),
            "{str_ip} must be allowed"
        );
    }
    Ok(())
}

#[test]
fn test_odd_ip_literals_are_normalized_then_refused() -> TestResult {
    let o_safety = UrlSafety::new(Arc::new(ProductionAddressPolicy::new()));
    for (str_url, str_normalized_host) in VEC_O_ODD_LITERALS.iter().copied() {
        let url_parsed = Url::parse(str_url)?;
        assert_eq!(
            url_parsed.host_str(),
            Some(str_normalized_host),
            "url crate normalization of {str_url}"
        );
        let res_check = o_safety.check(&url_parsed);
        assert!(
            matches!(res_check, Err(FetchError::BlockedAddress { .. })),
            "{str_url}: {res_check:?}"
        );
    }
    Ok(())
}

#[test]
fn test_ipv6_zone_ids_do_not_parse() {
    for str_url in [
        "http://[fe80::1%25en0]/",
        "http://[fe80::1%en0]/",
        "http://[::1%251]/",
    ] {
        assert!(
            Url::parse(str_url).is_err(),
            "{str_url} must be rejected by the URL parser"
        );
    }
}

#[test]
fn test_other_url_problems_are_refused() -> TestResult {
    let o_safety = UrlSafety::new(Arc::new(ProductionAddressPolicy::new()));
    let o_ftp = o_safety.check(&Url::parse("ftp://example.org/")?);
    assert!(matches!(o_ftp, Err(FetchError::UnsupportedScheme { .. })));
    let o_creds = o_safety.check(&Url::parse("https://u:p@example.org/")?);
    assert_eq!(o_creds, Err(FetchError::CredentialsInUrl));
    assert_eq!(
        o_safety.check(&Url::parse("https://example.org/a?b=c")?),
        Ok(())
    );
    Ok(())
}
