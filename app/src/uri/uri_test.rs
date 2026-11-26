use super::*;
use crate::ChannelState;

#[test]
fn validate_custom_uri_allows_retained_local_settings_pages() {
    let url = Url::parse(&format!(
        "{}://settings/appearance",
        ChannelState::url_scheme()
    ))
    .unwrap();
    assert!(validate_custom_uri(&url).is_ok());
}

#[test]
fn validate_custom_uri_allows_local_actions_without_server_config() {
    ChannelState::set(ChannelState::init());

    for url in [
        format!("{}://action/new_tab?path=/tmp", ChannelState::url_scheme()),
        format!("{}://settings/appearance", ChannelState::url_scheme()),
    ] {
        let url = Url::parse(&url).unwrap();
        assert!(
            validate_custom_uri(&url).is_ok(),
            "expected local URI to be valid: {url}"
        );
    }
}

// -- handle_incoming_uri redaction -------------------------------------------
//
// These tests cover the fix for GH #737: the entry log inside
// `handle_incoming_uri` used to write full URLs with secret query parameters
// to `warp.log` at `info` level before any redaction ran. They validate the
// redaction helper and the error messages produced by `validate_custom_uri` to
// ensure that the fallback `warn` emitted on invalid URIs never embeds the
// query string either.

/// The redacted log representation must contain scheme/host/path for triage
/// but must never contain the query string or any token material.
#[test]
fn safe_url_log_fields_redacts_refresh_token() {
    let url = Url::parse(&format!(
        "{}://mcp/oauth_callback?refresh_token=SENSITIVE_TOKEN&state=abc&user_uid=u",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(
        logged.contains(&format!("scheme={}", ChannelState::url_scheme())),
        "expected scheme in redacted log, got: {logged}"
    );
    assert!(
        logged.contains("host=mcp"),
        "expected host in redacted log, got: {logged}"
    );
    assert!(
        logged.contains("path=/oauth_callback"),
        "expected path in redacted log, got: {logged}"
    );
    assert!(
        !logged.contains("refresh_token"),
        "redacted log must not contain refresh_token: {logged}"
    );
    assert!(
        !logged.contains("SENSITIVE_TOKEN"),
        "redacted log must not contain the token value: {logged}"
    );
    assert!(
        !logged.contains("state="),
        "redacted log must not contain state query param: {logged}"
    );
    assert!(
        !logged.contains("user_uid"),
        "redacted log must not contain user_uid: {logged}"
    );
}

/// The redacted log representation must drop generic OAuth query parameters
/// (`code=`, `access_token=`, `custom_token=`, `token=`) regardless of host.
#[test]
fn safe_url_log_fields_redacts_generic_oauth_params() {
    let url = Url::parse(&format!(
        "{}://mcp/oauth_callback?code=AUTH_CODE&state=xyz&access_token=AT&custom_token=CT&token=RAW",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    for forbidden in [
        "code=",
        "AUTH_CODE",
        "access_token",
        "AT",
        "custom_token",
        "CT",
        "token=RAW",
        "state=",
    ] {
        assert!(
            !logged.contains(forbidden),
            "redacted log must not contain {forbidden:?}: {logged}"
        );
    }
    assert!(logged.contains("host=mcp"), "expected host: {logged}");
    assert!(
        logged.contains("path=/oauth_callback"),
        "expected path: {logged}"
    );
}

/// Local callback links can carry user-identifiable values in the query. The
/// entry log must not surface them on non-dogfood channels.
#[test]
fn safe_url_log_fields_redacts_user_email() {
    let url = Url::parse(&format!(
        "{}://mcp/oauth_callback?user_email=alice@example.com",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(
        !logged.contains("alice@example.com"),
        "redacted log must not contain invitee email: {logged}"
    );
    assert!(
        !logged.contains("user_email"),
        "redacted log must not contain user_email key: {logged}"
    );
    assert!(logged.contains("host=mcp"), "expected host: {logged}");
}

/// URL fragments are not currently used as secret carriers by Warp today, but
/// the entry log's contract is "scheme + host + path only", so fragments must
/// be dropped as well.
#[test]
fn safe_url_log_fields_drops_fragment() {
    let url = Url::parse(&format!(
        "{}://mcp/oauth_callback#sensitive_fragment",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(
        !logged.contains("sensitive_fragment"),
        "redacted log must not contain url fragment: {logged}"
    );
    assert!(
        !logged.contains('#'),
        "redacted log must not contain any fragment separator: {logged}"
    );
}

/// `file://` URLs route through the same entry log. `file://` URLs on macOS
/// have no host; the helper must not panic and must report `host=-` so the
/// format string stays well-formed.
#[test]
fn safe_url_log_fields_handles_file_urls_without_host() {
    let url = Url::parse("file:///tmp/foo.md").unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(logged.contains("scheme=file"), "expected scheme: {logged}");
    assert!(
        logged.contains("host=-"),
        "expected host placeholder: {logged}"
    );
    assert!(
        logged.contains("path=/tmp/foo.md"),
        "expected path: {logged}"
    );
}

/// `validate_custom_uri` returns `anyhow::Error`s whose messages feed the
/// non-dogfood `log::warn!("Custom URI is invalid: {e:?}")` fallback in
/// `handle_incoming_uri`. Those messages must never embed the full URL, its
/// query string, or its fragment — otherwise the fallback warn line becomes
/// a second secret leak.
#[test]
fn validate_custom_uri_errors_do_not_leak_query_string() {
    // Unexpected scheme.
    let url = Url::parse("https://mcp/oauth_callback?refresh_token=LEAKED").unwrap();
    let err = validate_custom_uri(&url).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.contains("refresh_token"), "{msg}");
    assert!(!msg.contains("LEAKED"), "{msg}");

    // Unexpected host.
    let url = Url::parse(&format!(
        "{}://unknown_host/desktop_redirect?refresh_token=LEAKED",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let err = validate_custom_uri(&url).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.contains("refresh_token"), "{msg}");
    assert!(!msg.contains("LEAKED"), "{msg}");

    // Unsupported path for a host that has a local-only page allowlist.
    let url = Url::parse(&format!(
        "{}://settings/not_the_page?refresh_token=LEAKED",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let err = validate_custom_uri(&url).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.contains("refresh_token"), "{msg}");
    assert!(!msg.contains("LEAKED"), "{msg}");
}

#[test]
fn test_parse_tab_path_expands_tilde() {
    let url = Url::parse("warp://action/new_tab?path=~/Projects").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home.join("Projects")));
}

#[test]
fn test_parse_tab_path_expands_url_encoded_tilde() {
    // `%7E` and `%2F` are URL-encoded `~` and `/`.
    let url = Url::parse("warp://action/new_tab?path=%7E%2FProjects").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home.join("Projects")));
}

#[test]
fn test_parse_tab_path_absolute_path_unchanged() {
    let url = Url::parse("warp://action/new_tab?path=/tmp/foo").unwrap();
    assert_eq!(parse_tab_path(&url), Some(PathBuf::from("/tmp/foo")));
}

#[test]
fn test_parse_tab_path_relative_path_unchanged() {
    let url = Url::parse("warp://action/new_tab?path=relative/dir").unwrap();
    assert_eq!(parse_tab_path(&url), Some(PathBuf::from("relative/dir")));
}

#[test]
fn test_parse_tab_path_missing_returns_none() {
    let url = Url::parse("warp://action/new_tab").unwrap();
    assert_eq!(parse_tab_path(&url), None);
}

#[test]
fn test_parse_tab_path_bare_tilde() {
    let url = Url::parse("warp://action/new_tab?path=~").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home));
}

#[test]
fn removed_agent_deeplinks_are_rejected() {
    let url = Url::parse(&format!(
        "{}://action/new_agent_conversation",
        ChannelState::url_scheme()
    ))
    .unwrap();
    assert!(Action::parse(&url).is_err());
    let settings_url =
        Url::parse(&format!("{}://settings/mcp", ChannelState::url_scheme())).unwrap();
    assert!(validate_custom_uri(&settings_url).is_err());
    for host in ["mcp", "linear"] {
        let url = Url::parse(&format!("{}://{host}/", ChannelState::url_scheme())).unwrap();
        assert!(validate_custom_uri(&url).is_err());
    }
}

#[test]
fn removed_launch_and_repository_uris_are_rejected() {
    assert!(UriHost::from_str("launch").is_err());
    let url = Url::parse(&format!(
        "{}://action/open-repo",
        ChannelState::url_scheme()
    ))
    .unwrap();
    assert!(Action::parse(&url).is_err());
}
