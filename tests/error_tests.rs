extern crate atom;

#[test]
fn test_error() {
    let err = atom::error::Error::new("test error", "test file.at", 5, 7);
    assert_eq!(err.message(), "test error");
    assert_eq!(err.file_name(), "test file.at");
    assert_eq!(err.line(), 5);
    assert_eq!(err.column(), 7);
}
