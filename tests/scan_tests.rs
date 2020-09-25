extern crate atom;

use atom::error::*;
use atom::scan::*;

fn verify_token(actual: &Token, expected: &Token, ignore_pos: bool) {
    assert_eq!(actual.token_type(), expected.token_type());
    assert_eq!(actual.token_data(), expected.token_data());
    assert_eq!(actual.source_name(), expected.source_name());
    if !ignore_pos {
        assert_eq!(actual.source_line(), expected.source_line());
        assert_eq!(actual.source_column(), expected.source_column());
    }
}

fn verify_error_at(scanner: &mut Scanner, expected: &Error) {
    for next in scanner {
        if let Err(e) = next {
            // Make sure that this is the error we expected to get. The
            // exact error message is ignored.
            assert_eq!(e.file_name(), expected.file_name());
            assert_eq!(e.line(), expected.line());
            assert_eq!(e.column(), expected.column());
            // Once we found the error that we where looking for
            // we can move on.
            return;
        }
    }

    panic!("did not find a lexer error");
}

fn verify_list(scanner: &mut Scanner, expected: &Vec<Token>, ignore_pos: bool) {
    for tok in expected {
        let actual = scanner.next().unwrap();
        match actual {
            Ok(a) => verify_token(&a, &tok, ignore_pos),
            Err(e) => panic!(e),
        }
    }

    // Make sure that there are no left over tokens.
    if scanner.next().is_some() {
        panic!("scanner left over tokens");
    }
}

#[test]
fn test_token() {
    let t = Token::new(TokenType::NumberLiteral, "source file", 5, 3, "578");
    assert_eq!(t.token_type(), TokenType::NumberLiteral);
    assert_eq!(t.source_name(), "source file");
    assert_eq!(t.source_line(), 5);
    assert_eq!(t.source_column(), 3);
    assert_eq!(t.token_data(), "578");
}

#[test]
fn test_scanner_starting_state() {
    let scan = Scanner::new("test", "");
    assert_eq!(scan.source_name(), "test");
    assert_eq!(scan.current_line(), 1);
    assert_eq!(scan.current_column(), 0);
}

#[test]
fn test_operators() {
    let mut scanner = Scanner::new("test", "+ - * / ! | & ^ % ~ .");
    verify_list(
        &mut scanner,
        &vec![
            Token::new(TokenType::Plus, "test", 1, 1, "+"),
            Token::new(TokenType::Minus, "test", 1, 3, "-"),
            Token::new(TokenType::Star, "test", 1, 5, "*"),
            Token::new(TokenType::Slash, "test", 1, 7, "/"),
            Token::new(TokenType::Bang, "test", 1, 9, "!"),
            Token::new(TokenType::Pipe, "test", 1, 11, "|"),
            Token::new(TokenType::Ampersand, "test", 1, 13, "&"),
            Token::new(TokenType::Caret, "test", 1, 15, "^"),
            Token::new(TokenType::Percent, "test", 1, 17, "%"),
            Token::new(TokenType::Tilde, "test", 1, 19, "~"),
            Token::new(TokenType::Dot, "test", 1, 21, "."),
        ],
        false,
    );
}

#[test]
fn test_comments() {
    let mut scanner = Scanner::new("test", "\t// This is a comment and should be ignored\n5");
    verify_list(
        &mut scanner,
        &vec![Token::new(TokenType::NumberLiteral, "test", 2, 1, "5")],
        false,
    );

    scanner.provide("test", "5 // 5\n// 5");
    verify_list(
        &mut scanner,
        &vec![Token::new(TokenType::NumberLiteral, "test", 1, 1, "5")],
        false,
    );

    scanner.provide("test", "5 /* this \nshould \treally not \nmatter\n***/3");
    verify_list(
        &mut scanner,
        &vec![
            Token::new(TokenType::NumberLiteral, "test", 1, 1, "5"),
            Token::new(TokenType::NumberLiteral, "test", 4, 5, "3"),
        ],
        false,
    );

    scanner.provide("test", "/*");
    verify_error_at(&mut scanner, &Error::new("", "test", 1, 3));
}

#[test]
fn test_numbers() {
    let mut scanner = Scanner::new("test", "5");
    verify_list(
        &mut scanner,
        &vec![Token::new(TokenType::NumberLiteral, "test", 1, 1, "5")],
        false,
    );

    scanner.provide("test", "7.8");
    verify_list(
        &mut scanner,
        &vec![Token::new(TokenType::NumberLiteral, "test", 1, 1, "7.8")],
        false,
    );

    scanner.provide("test", "\t4 67.3 \n\t .01 156793530 \t");
    verify_list(
        &mut scanner,
        &vec![
            Token::new(TokenType::NumberLiteral, "test", 1, 2, "4"),
            Token::new(TokenType::NumberLiteral, "test", 1, 4, "67.3"),
            Token::new(TokenType::NumberLiteral, "test", 2, 3, ".01"),
            Token::new(TokenType::NumberLiteral, "test", 2, 7, "156793530"),
        ],
        false,
    );
}

#[test]
fn test_strings() {
    let mut scanner: Scanner = Scanner::new("test", "'' \"\"");
    verify_list(
        &mut scanner,
        &vec![
            Token::new(TokenType::StringLiteral, "test", 1, 1, ""),
            Token::new(TokenType::FormattedStringLiteral, "test", 1, 4, ""),
        ],
        false,
    );

    scanner.provide("test", "'hello world'");
    verify_list(
        &mut scanner,
        &vec![Token::new(
            TokenType::StringLiteral,
            "test",
            1,
            1,
            "hello world",
        )],
        false,
    );

    scanner.provide("test", "'hello\\nworld'");
    verify_list(
        &mut scanner,
        &vec![Token::new(
            TokenType::StringLiteral,
            "test",
            1,
            1,
            "hello\nworld",
        )],
        false,
    );

    scanner.provide("test", "'\\t\\n\\r\\\\'");
    verify_list(
        &mut scanner,
        &vec![Token::new(
            TokenType::StringLiteral,
            "test",
            1,
            1,
            "\t\n\r\\",
        )],
        false,
    );

    // Verify that errors are returned when invalid strings are given.
    scanner.provide("test", "'");
    verify_error_at(&mut scanner, &Error::new("", "test", 1, 2));
    scanner.provide("test", "'\\");
    verify_error_at(&mut scanner, &Error::new("", "test", 1, 3));
    scanner.provide("test", "'\"");
    verify_error_at(&mut scanner, &Error::new("", "test", 1, 3));
    scanner.provide("test", "'\\a'");
    verify_error_at(&mut scanner, &Error::new("", "test", 1, 3));
}

#[test]
fn test_identifier() {
    let mut scanner: Scanner =
        Scanner::new("test", "hello world testing123 test_123 hello_world _");
    verify_list(
        &mut scanner,
        &vec![
            Token::new(TokenType::Identifier, "test", 1, 1, "hello"),
            Token::new(TokenType::Identifier, "test", 1, 7, "world"),
            Token::new(TokenType::Identifier, "test", 1, 13, "testing123"),
            Token::new(TokenType::Identifier, "test", 1, 24, "test_123"),
            Token::new(TokenType::Identifier, "test", 1, 33, "hello_world"),
            Token::new(TokenType::Identifier, "test", 1, 45, "_"),
        ],
        false,
    );
}
