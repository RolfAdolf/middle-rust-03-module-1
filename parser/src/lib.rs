mod bin_format;
mod common;
mod constant;
mod csv_format;
mod error;
mod parser;
mod record;
mod txt_format;

use bin_format::{BinParser, YPBankBinRecordParser};
use csv_format::{CsvParser, YPBankCsvRecordParser};
use parser::Parser;
use txt_format::{TxtParser, YPBankTxtRecordParser};

pub use common::{Format, TransactionStatus, TransactionType};
pub use error::ParseError;
pub use record::YPBankRecord;

/// A unified parser that can read and write bank records in multiple formats - CSV, TXT, and binary.
///
/// # Examples
///
/// ```no_run
/// use parser::{CommonParser, Format};
/// use std::fs::File;
///
/// let parser = CommonParser::new(Format::Csv);
/// let mut file = File::open("records.csv").unwrap();
/// let records = parser.from_read(&mut file).unwrap();
/// ```
pub struct CommonParser {
    format: Format,
}

impl CommonParser {
    pub fn new(format: Format) -> Self {
        Self { format }
    }
}

impl CommonParser {
    /// Reads and parses all records from a file.
    ///
    /// This method reads the entire file and returns a vector of parsed records.
    /// Format-specific headers and structure are handled automatically.
    ///
    /// # Arguments
    ///
    /// * `r` - A readable source (file, buffer, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<YPBankRecord>)` - Successfully parsed records
    /// * `Err(ParseError)` - If parsing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use parser::{CommonParser, Format};
    /// use std::fs::File;
    ///
    /// let parser = CommonParser::new(Format::Csv);
    /// let mut file = File::open("records.csv").unwrap();
    /// let records = parser.from_read(&mut file).unwrap();
    /// ```
    pub fn from_read<Reader: std::io::Read>(
        &self,
        r: &mut Reader,
    ) -> Result<Vec<YPBankRecord>, ParseError> {
        match self.format {
            Format::Csv => <CsvParser as Parser<YPBankCsvRecordParser>>::from_read(r),
            Format::Txt => <TxtParser as Parser<YPBankTxtRecordParser>>::from_read(r),
            Format::Bin => <BinParser as Parser<YPBankBinRecordParser>>::from_read(r),
        }
    }

    /// Writes records to a file in the parser's format.
    ///
    /// This method writes all records to the specified writer, including
    /// format-specific headers and structure.
    ///
    /// # Arguments
    ///
    /// * `w` - A writable destination (file, buffer, stdout, etc.)
    /// * `records` - Vector of records to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully written
    /// * `Err(ParseError)` - If writing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use parser::{CommonParser, Format, YPBankRecord};
    /// use std::io::stdout;
    ///
    /// let parser = CommonParser::new(Format::Txt);
    /// let records = vec![/* ... */];
    /// parser.write_to(&mut stdout(), &records).unwrap();
    /// ```
    pub fn write_to<Writer: std::io::Write>(
        &self,
        w: &mut Writer,
        records: &Vec<YPBankRecord>,
    ) -> Result<(), ParseError> {
        match self.format {
            Format::Csv => <CsvParser as Parser<YPBankCsvRecordParser>>::write_to(w, records),
            Format::Txt => <TxtParser as Parser<YPBankTxtRecordParser>>::write_to(w, records),
            Format::Bin => <BinParser as Parser<YPBankBinRecordParser>>::write_to(w, records),
        }
    }
}
