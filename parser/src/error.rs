use crate::common::TransactionType;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

/// Errors that can occur during parsing or writing of bank records.
///
/// This enum covers all possible error conditions when working with bank
/// transaction records, including I/O errors, format errors, and validation errors.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    IOError(String),
    InvalidTransactionTypeValue(String),
    InvalidStatusValue(String),
    InvalidUserId(String, TransactionType),
    InvalidRawValue(String),
    InvalidRow(String),
    InvalidCsvHeader(String),
    UnexpectedEOF,
    FieldNotFound(String),
    InconsistentRecord(String),
    InvalidMagic(String),
    InvalidFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::IOError(ref msg) => write!(f, "Read error: {}", msg),
            ParseError::InvalidTransactionTypeValue(ref msg) => {
                write!(f, "Invalid transaction type value found: {}", msg)
            }
            ParseError::InvalidStatusValue(ref msg) => {
                write!(f, "Invalid status value found: {}", msg)
            }
            ParseError::InvalidUserId(ref user_id, ref transaction_type) => write!(
                f,
                "Invalid user id {} for transaction type {}",
                user_id,
                transaction_type.as_str()
            ),
            ParseError::InvalidRawValue(ref msg) => write!(f, "Invalid raw value found: {}", msg),
            ParseError::InvalidRow(ref msg) => write!(f, "Invalid row found: {}", msg),
            ParseError::InvalidCsvHeader(ref msg) => write!(f, "Invalid CSV header: {}", msg),
            ParseError::UnexpectedEOF => write!(f, "Unexpected EOF"),
            ParseError::FieldNotFound(ref msg) => write!(f, "Value is not set for field: {}", msg),
            ParseError::InconsistentRecord(ref msg) => {
                write!(f, "Inconsistent record found: {}", msg)
            }
            ParseError::InvalidMagic(ref msg) => write!(f, "Invalid magic found: {}", msg),
            ParseError::InvalidFormat(ref msg) => write!(f, "Invalid file format found: {}", msg),
        }
    }
}

impl Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IOError(err.to_string())
    }
}
