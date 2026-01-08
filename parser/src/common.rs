use crate::constant::{DEPOSIT, TRANSFER, WITHDRAWAL};
use crate::constant::{FAILURE, PENDING, SUCCESS};
use crate::error::ParseError;
use crate::error::ParseError::{InvalidStatusValue, InvalidTransactionTypeValue};
use std::io::BufRead;

/// Supported file formats for bank transaction records.
///
/// This enum represents the three formats that can be used to store and read
/// bank transaction records: CSV, TXT (text), and binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Csv,
    Txt,
    Bin,
}

impl Format {
    /// Returns the string representation of the format.
    ///
    /// # Returns
    ///
    /// * `"csv"` for CSV format
    /// * `"txt"` for TXT format
    /// * `"binary"` for binary format
    ///
    /// # Examples
    ///
    /// ```
    /// use parser::Format;
    ///
    /// assert_eq!(Format::Csv.as_str(), "csv");
    /// assert_eq!(Format::Bin.as_str(), "binary");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Csv => "csv",
            Format::Txt => "txt",
            Format::Bin => "binary",
        }
    }
}

impl std::str::FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(Format::Csv),
            "txt" => Ok(Format::Txt),
            "binary" => Ok(Format::Bin),
            _ => Err(ParseError::InvalidFormat(s.to_string())),
        }
    }
}

/// Type of bank transaction.
///
/// Represents the three possible transaction types in the banking system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Deposit => DEPOSIT,
            TransactionType::Transfer => TRANSFER,
            TransactionType::Withdrawal => WITHDRAWAL,
        }
    }

    pub fn from_int(val: u8) -> Result<Self, ParseError> {
        match val {
            0 => Ok(TransactionType::Deposit),
            1 => Ok(TransactionType::Transfer),
            2 => Ok(TransactionType::Withdrawal),
            _ => Err(InvalidTransactionTypeValue(val.to_string())),
        }
    }

    pub fn as_int(&self) -> u8 {
        match self {
            TransactionType::Deposit => 0,
            TransactionType::Transfer => 1,
            TransactionType::Withdrawal => 2,
        }
    }
}

impl std::str::FromStr for TransactionType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            DEPOSIT => Ok(TransactionType::Deposit),
            TRANSFER => Ok(TransactionType::Transfer),
            WITHDRAWAL => Ok(TransactionType::Withdrawal),
            _ => Err(InvalidTransactionTypeValue(s.to_string())),
        }
    }
}

/// Status of a bank transaction.
///
/// Represents the three possible states a transaction can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Success,
    Failure,
    Pending,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Success => SUCCESS,
            TransactionStatus::Failure => FAILURE,
            TransactionStatus::Pending => PENDING,
        }
    }

    pub fn from_int(val: u8) -> Result<Self, ParseError> {
        match val {
            0 => Ok(TransactionStatus::Success),
            1 => Ok(TransactionStatus::Failure),
            2 => Ok(TransactionStatus::Pending),
            _ => Err(InvalidStatusValue(val.to_string())),
        }
    }

    pub fn as_int(&self) -> u8 {
        match self {
            TransactionStatus::Success => 0,
            TransactionStatus::Failure => 1,
            TransactionStatus::Pending => 2,
        }
    }
}

impl std::str::FromStr for TransactionStatus {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            SUCCESS => Ok(TransactionStatus::Success),
            FAILURE => Ok(TransactionStatus::Failure),
            PENDING => Ok(TransactionStatus::Pending),
            _ => Err(InvalidStatusValue(s.to_string())),
        }
    }
}

pub fn parse_value_from_string<T: std::str::FromStr>(s: String) -> Result<T, ParseError> {
    match s.parse::<T>() {
        Ok(v) => Ok(v),
        Err(_) => Err(ParseError::InvalidRawValue(s)),
    }
}

pub fn parse_from_user_id(s: String, transaction_type: TransactionType) -> Result<u64, ParseError> {
    let val = s
        .parse::<u64>()
        .map_err(|_| ParseError::InvalidRawValue(s))?;

    validate_from_user_id(val, transaction_type)
}

pub fn parse_to_user_id(s: String, transaction_type: TransactionType) -> Result<u64, ParseError> {
    let val = s
        .parse::<u64>()
        .map_err(|_| ParseError::InvalidRawValue(s))?;

    validate_to_user_id(val, transaction_type)
}

pub fn validate_from_user_id(
    val: u64,
    transaction_type: TransactionType,
) -> Result<u64, ParseError> {
    if val == 0 && transaction_type != TransactionType::Deposit {
        return Err(ParseError::InvalidUserId(val.to_string(), transaction_type));
    }

    Ok(val)
}

pub fn validate_to_user_id(val: u64, transaction_type: TransactionType) -> Result<u64, ParseError> {
    if val == 0 && transaction_type != TransactionType::Withdrawal {
        return Err(ParseError::InvalidUserId(val.to_string(), transaction_type));
    }

    Ok(val)
}

macro_rules! impl_read_from_bytes {
    ($name:ident, $type:ty, $size:expr) => {
        pub fn $name<R: BufRead>(r: &mut R) -> Result<$type, ParseError> {
            let mut bytes = [0; $size];
            r.read_exact(&mut bytes)?;
            Ok(<$type>::from_be_bytes(bytes))
        }
    };
}

impl_read_from_bytes!(read_u64_from_bytes, u64, 8);
impl_read_from_bytes!(read_u8_from_bytes, u8, 1);
impl_read_from_bytes!(read_i64_from_bytes, i64, 8);
impl_read_from_bytes!(read_u32_from_bytes, u32, 4);
