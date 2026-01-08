use crate::common::parse_value_from_string;
use crate::common::{TransactionType, parse_from_user_id, parse_to_user_id};
use crate::error::ParseError;
use crate::parser::{Parser, YPBankRecordParser};
use crate::record::YPBankRecord;
use std::str::FromStr;

const SEP: char = ',';
const QUOTE: char = '"';
const TARGET_HEADER: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";

struct Separator {
    line: String,
    index: usize,
    is_inside_quotes: bool,
}

impl Separator {
    fn new(line: String) -> Self {
        Separator {
            line,
            index: 0,
            is_inside_quotes: false,
        }
    }
}

impl Iterator for Separator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.line.len() {
            return None;
        }

        let start = self.index;
        let mut field_end = start;

        for (byte_pos, ch) in self.line.char_indices().skip_while(|(i, _)| *i < start) {
            if !self.is_inside_quotes && ch == SEP {
                self.index = byte_pos + ch.len_utf8();
                return Some(self.line[start..field_end].to_string());
            }

            if ch == QUOTE {
                self.is_inside_quotes = !self.is_inside_quotes;
            }
            field_end = byte_pos + ch.len_utf8();
        }

        let result = self.line[start..field_end].to_string();
        self.index = self.line.len();
        Some(result)
    }
}

pub struct YPBankCsvRecordParser {}

impl YPBankCsvRecordParser {
    fn from_raw_values(raw_values: Vec<String>) -> Result<YPBankRecord, ParseError> {
        if raw_values.len() != 8 {
            return Err(ParseError::InvalidRow(format!(
                "Expected 8 fields, got {}",
                raw_values.len()
            )));
        }

        let tt_parse_result = TransactionType::from_str(&raw_values[1])?;

        Ok(YPBankRecord::new(
            parse_value_from_string(raw_values[0].clone())?,
            parse_value_from_string(raw_values[1].clone())?,
            parse_from_user_id(raw_values[2].clone(), tt_parse_result)?,
            parse_to_user_id(raw_values[3].clone(), tt_parse_result)?,
            parse_value_from_string(raw_values[4].clone())?,
            parse_value_from_string(raw_values[5].clone())?,
            parse_value_from_string(raw_values[6].clone())?,
            raw_values[7].clone(),
        ))
    }
}

impl YPBankRecordParser for YPBankCsvRecordParser {
    fn from_read<R: std::io::BufRead>(r: &mut R) -> Result<Option<YPBankRecord>, ParseError> {
        let mut line = String::new();
        let bytes_read = r.read_line(&mut line)?;

        if bytes_read == 0 || line.trim().is_empty() {
            return Ok(None);
        }

        let sep = Separator::new(line.trim().to_string());
        let mut values = vec![];
        for value in sep {
            values.push(value);
        }

        let record = Self::from_raw_values(values)?;
        Ok(Some(record))
    }

    fn write_to<W: std::io::Write>(record: &YPBankRecord, w: &mut W) -> Result<(), ParseError> {
        let record_str = format!(
            "{},{},{},{},{},{},{},{}\n",
            record.id,
            record.transaction_type.as_str(),
            record.from_user_id,
            record.to_user_id,
            record.amount,
            record.ts,
            record.status.as_str(),
            record.description
        );

        w.write_all(record_str.as_bytes())?;
        Ok(())
    }
}

pub struct CsvParser {}

impl Parser<YPBankCsvRecordParser> for CsvParser {
    fn pre_read<R: std::io::BufRead>(r: &mut R) -> Result<(), ParseError> {
        let mut line = String::new();

        r.read_line(&mut line)?;

        if line != TARGET_HEADER {
            return Err(ParseError::InvalidCsvHeader(line));
        }

        Ok(())
    }

    fn pre_write<W: std::io::Write>(w: &mut W) -> Result<(), ParseError> {
        w.write_all(TARGET_HEADER.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod separator_tests {
    use super::*;

    #[test]
    fn test_regular_case() {
        let test_line = "val1,val 2, val 3 ".to_string();
        let target_values = vec!["val1", "val 2", " val 3 "];

        let sep = Separator::new(test_line);

        let result = sep.collect::<Vec<String>>();
        assert_eq!(result, target_values);
    }

    #[test]
    fn test_quotes() {
        let test_line = "val1,val 2, \" val,,,3 \" ".to_string();
        let target_values = vec!["val1", "val 2", " \" val,,,3 \" "];

        let sep = Separator::new(test_line);

        let result = sep.collect::<Vec<String>>();
        assert_eq!(result, target_values);
    }

    #[test]
    fn test_empty_line() {
        let test_line = "".to_string();
        let target_values: Vec<String> = vec![];

        let sep = Separator::new(test_line);

        let result = sep.collect::<Vec<String>>();
        assert_eq!(result, target_values);
    }

    #[test]
    fn test_empty_value_in_line() {
        let test_line = "val1,,val3".to_string();
        let target_values = vec!["val1", "", "val3"];

        let sep = Separator::new(test_line);

        let result = sep.collect::<Vec<String>>();
        assert_eq!(result, target_values);
    }
}

#[cfg(test)]
mod yp_bank_csv_record_tests {
    use super::*;
    use crate::common::TransactionStatus;
    use std::io::Cursor;

    #[test]
    fn test_from_read_regular_case() {
        let raw_line = "1000000000000000,DEPOSIT,1,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n";
        let mut reader = Cursor::new(raw_line.as_bytes());

        let target_record = YPBankRecord::new(
            1000000000000000,
            TransactionType::Deposit,
            1,
            9223372036854775807,
            100,
            1633036860000,
            TransactionStatus::Failure,
            "\"Record number 1\"".to_string(),
        );

        let result = YPBankCsvRecordParser::from_read(&mut reader);

        assert!(result.is_ok(), "Parsing should succeed");
        let record_opt = result.expect("Should parse successfully");
        assert!(record_opt.is_some(), "Should return Some(record)");
        assert_eq!(record_opt.expect("Should have a record"), target_record);
    }

    #[test]
    fn test_from_read_invalid_from_user_id() {
        let raw_line = "1000000000000000,TRANSFER,0,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n";
        let mut reader = Cursor::new(raw_line.as_bytes());

        let result = YPBankCsvRecordParser::from_read(&mut reader);

        assert!(result.is_err(), "Should return an error");

        let error = result.err().expect("Should return an error");
        assert_eq!(
            error,
            ParseError::InvalidUserId("0".to_string(), TransactionType::Transfer)
        );
    }

    #[test]
    fn test_from_read_invalid_to_user_id() {
        let raw_line =
            "1000000000000000,TRANSFER,1,0,100,1633036860000,FAILURE,\"Record number 1\"\n";
        let mut reader = Cursor::new(raw_line.as_bytes());

        let result = YPBankCsvRecordParser::from_read(&mut reader);

        assert!(result.is_err(), "Should return an error");

        let error = result.err().expect("Should return an error");
        assert_eq!(
            error,
            ParseError::InvalidUserId("0".to_string(), TransactionType::Transfer)
        );
    }

    #[test]
    fn test_from_read_eof() {
        let mut reader = Cursor::new(Vec::<u8>::new());
        let result = YPBankCsvRecordParser::from_read(&mut reader);

        assert!(result.is_ok(), "EOF should return Ok(None)");
        assert!(
            result.expect("Should parse successfully").is_none(),
            "Should return None on EOF"
        );
    }

    #[test]
    fn test_from_read_empty_line() {
        let raw_line = "\n";
        let mut reader = Cursor::new(raw_line.as_bytes());

        let result = YPBankCsvRecordParser::from_read(&mut reader);

        assert!(result.is_ok(), "Empty line should return Ok(None)");
        assert!(
            result.expect("Should parse successfully").is_none(),
            "Should return None on empty line"
        );
    }

    #[test]
    fn test_write_to_regular_case() {
        let record = YPBankRecord::new(
            1000000000000000,
            TransactionType::Deposit,
            1,
            9223372036854775807,
            100,
            1633036860000,
            TransactionStatus::Failure,
            "\"Record number 1\"".to_string(),
        );
        let target_result = "1000000000000000,DEPOSIT,1,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n";

        let mut writer = Cursor::new(Vec::new());
        let result = YPBankCsvRecordParser::write_to(&record, &mut writer);
        assert!(result.is_ok(), "Writing should succeed");

        let written =
            String::from_utf8(writer.into_inner()).expect("Written data should be valid UTF-8");
        assert_eq!(written, target_result);
    }
}

#[cfg(test)]
mod csv_parser_tests {
    use super::*;
    use crate::common::TransactionStatus;

    #[test]
    fn test_from_read() {
        let raw_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1000000000000000,DEPOSIT,1,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n1000000000000001,TRANSFER,1,9223372036854775807,200,1633036860000,PENDING,\"Record number 2\"\n";
        let target_records: Vec<YPBankRecord> = vec![
            YPBankRecord::new(
                1000000000000000,
                TransactionType::Deposit,
                1,
                9223372036854775807,
                100,
                1633036860000,
                TransactionStatus::Failure,
                "\"Record number 1\"".to_string(),
            ),
            YPBankRecord::new(
                1000000000000001,
                TransactionType::Transfer,
                1,
                9223372036854775807,
                200,
                1633036860000,
                TransactionStatus::Pending,
                "\"Record number 2\"".to_string(),
            ),
        ];

        let mut reader = std::io::Cursor::new(raw_data.as_bytes());
        let records = CsvParser::from_read(&mut reader).expect("Should parse successfully");
        assert_eq!(records.len(), 2);

        assert_eq!(records, target_records);
    }

    #[test]
    fn test_write_to() {
        let records = vec![
            YPBankRecord::new(
                1000000000000000,
                TransactionType::Deposit,
                1,
                9223372036854775807,
                100,
                1633036860000,
                TransactionStatus::Failure,
                "\"Record number 1\"".to_string(),
            ),
            YPBankRecord::new(
                1000000000000001,
                TransactionType::Transfer,
                1,
                9223372036854775807,
                200,
                1633036860000,
                TransactionStatus::Pending,
                "\"Record number 2\"".to_string(),
            ),
        ];
        let raw_data = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1000000000000000,DEPOSIT,1,9223372036854775807,100,1633036860000,FAILURE,\"Record number 1\"\n1000000000000001,TRANSFER,1,9223372036854775807,200,1633036860000,PENDING,\"Record number 2\"\n";

        let mut writer = std::io::Cursor::new(Vec::new());
        CsvParser::write_to(&mut writer, &records).expect("Should write successfully");
        let result = writer.into_inner();
        assert_eq!(result, raw_data.as_bytes());
    }
}
