extern crate atom;

use atom::scan::*;
use atom::*;

#[test]
fn test_token() {
    let t = Token::NumberLiteral(TokenData::new("source file", 5, 3, "578"));
    match t {
        Token::NumberLiteral(val) => {
            assert_eq!(val.source_name(), "source file");
            assert_eq!(val.source_line(), 5);
            assert_eq!(val.source_column(), 3);
            assert_eq!(val.data(), "578");
        }
        _ => {
            assert_eq!(true, false);
        }
    }
}

fn compare_tokens(scanner: &Scanner, expected: &Vec<Token>, bool ignorePos) {
    for t in expected {
        if let Some(actual) = scanner.next() {
            assert_eq!(actual, expected);
        }
    }
    // Make sure that the scanner did not find more tokens
    // then what there really are.
    assert_ne!(scanner.next(), None);
}

#[test]
fn test_scanner_starting_state() {
    let scan = Scanner::new("test", "");
    assert_eq!(scan.source_name(), "test");
    assert_eq!(scan.current_line(), 1);
    assert_eq!(scan.current_column(), 1);
}