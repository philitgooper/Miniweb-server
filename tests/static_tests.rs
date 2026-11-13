use miniweb_server::{http::Status, static_files::StaticFiles};
use std::fs;
#[test]
fn serves_index_and_blocks_traversal() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("index.html"), "hello").unwrap();
    let outside = dir.path().parent().unwrap().join("miniweb-test-secret");
    fs::write(&outside, "secret").unwrap();
    let files = StaticFiles::new(dir.path(), 1024).unwrap();
    assert_eq!(files.get("/").status.code, 200);
    assert_eq!(
        files.get("/%2e%2e/miniweb-test-secret").status,
        Status::NOT_FOUND
    );
    let _ = fs::remove_file(outside);
}
