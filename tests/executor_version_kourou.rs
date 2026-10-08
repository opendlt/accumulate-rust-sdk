use accumulate_client::ExecutorVersion;

#[test]
fn kourou_executor_version_roundtrips() {
    let k: ExecutorVersion = serde_json::from_str("\"v2-kourou\"").unwrap();
    assert_eq!(k, ExecutorVersion::V2Kourou);
    let k2: ExecutorVersion = serde_json::from_str("\"v2Kourou\"").unwrap();
    assert_eq!(k2, ExecutorVersion::V2Kourou);
    assert_eq!(serde_json::to_string(&k).unwrap(), "\"v2Kourou\"");
}
