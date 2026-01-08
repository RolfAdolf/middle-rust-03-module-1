use crate::common::parse_value_from_string;
use crate::common::{TransactionType, parse_from_user_id, parse_to_user_id};
use crate::error::ParseError;
use crate::parser::{Parser, YPBankRecordParser};
use crate::record::YPBankRecord;
use std::collections::HashMap;
use std::iter::zip;
use std::str::FromStr;

const SEP: char = ':';
const COMMENT_PREFIX: char = '#';
const NEW_LINE: char = '\n';

pub struct YPBankTxtRecordParser {}

impl YPBankTxtRecordParser {
    const FIELDS: [&str; 8] = [
        "TX_ID",
        "TX_TYPE",
        "FROM_USER_ID",
        "TO_USER_ID",
        "AMOUNT",
        "TIMESTAMP",
        "STATUS",
        "DESCRIPTION",
    ];

    fn parse_raw_values<R: std::io::BufRead>(
        r: &mut R,
    ) -> Result<Option<HashMap<String, String>>, ParseError> {
        let mut raw_values = HashMap::<String, String>::new();

        let mut parsed_values = 0;
        while parsed_values < 8 {
            let mut line = String::new();
            let res = r.read_line(&mut line)?;

            if res == 0 {
                if parsed_values == 0 {
                    return Ok(None);
                }

                return Err(ParseError::InconsistentRecord(
                    "unexpected end of file while parsing".to_string(),
                ));
            }

            if line.starts_with(COMMENT_PREFIX) {
                continue;
            }

            if line == NEW_LINE.to_string() {
                if parsed_values == 0 {
                    continue;
                }

                return Err(ParseError::InconsistentRecord(
                    "unexpected new line while parsing".to_string(),
                ));
            }

            let [key, val] = Self::parse_raw_line(line)?;
            raw_values.insert(key, val);
            parsed_values += 1
        }

        Ok(Some(raw_values))
    }

    fn from_raw_values(values_map: HashMap<String, String>) -> Result<YPBankRecord, ParseError> {
        let mut values: Vec<String> = vec![];
        for field in Self::FIELDS {
            match values_map.get(field) {
                None => return Err(ParseError::FieldNotFound(field.to_string())),
                Some(value) => {
                    values.push(value.to_string());
                }
            };
        }

        let tt_parse_result = TransactionType::from_str(&values[1])?;

        Ok(YPBankRecord::new(
            parse_value_from_string(values[0].clone())?,
            parse_value_from_string(values[1].clone())?,
            parse_from_user_id(values[2].clone(), tt_parse_result)?,
            parse_to_user_id(values[3].clone(), tt_parse_result)?,
            parse_value_from_string(values[4].clone())?,
            parse_value_from_string(values[5].clone())?,
            parse_value_from_string(values[6].clone())?,
            values[7].clone(),
        ))
    }

    fn parse_raw_line(line: String) -> Result<[String; 2], ParseError> {
        let parts = line.split(SEP).collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(ParseError::InvalidRow(line));
        }

        Ok([parts[0].trim().to_string(), parts[1].trim().to_string()])
    }
}

impl YPBankRecordParser for YPBankTxtRecordParser {
    fn from_read<R: std::io::BufRead>(r: &mut R) -> Result<Option<YPBankRecord>, ParseError> {
        let raw_values_opt = Self::parse_raw_values(r)?;

        match raw_values_opt {
            Some(raw_values) => {
                let record = Self::from_raw_values(raw_values)?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    fn write_to<W: std::io::Write>(record: &YPBankRecord, w: &mut W) -> Result<(), ParseError> {
        let record_values = [
            record.id.to_string(),
            record.transaction_type.as_str().to_string(),
            record.from_user_id.to_string(),
            record.to_user_id.to_string(),
            record.amount.to_string(),
            record.ts.to_string(),
            record.status.as_str().to_string(),
            record.description.to_string(),
        ];

        let mut raw_values: Vec<String> = vec![];
        for (key, val) in zip(Self::FIELDS.iter(), &record_values) {
            raw_values.push(format!("{}: {}", key, val));
        }
        raw_values.push(NEW_LINE.to_string());

        let result = raw_values.join(NEW_LINE.to_string().as_str());

        w.write_all(result.as_bytes())?;

        Ok(())
    }
}

pub struct TxtParser {}

impl Parser<YPBankTxtRecordParser> for TxtParser {}

#[cfg(test)]
mod yp_bank_txt_record_tests {
    use super::*;
    use crate::common::TransactionStatus;
    use std::io::Cursor;

    #[test]
    fn test_from_read_regular_case() {
        let raw_data = "# Record 1 (DEPOSIT)\nTX_TYPE: DEPOSIT\nTO_USER_ID: 9223372036854775807\nFROM_USER_ID: 0\nTIMESTAMP: 1633036860000\nDESCRIPTION: \"Record number 1\"\nTX_ID: 1000000000000000\nAMOUNT: 100\nSTATUS: FAILURE\n";
        let mut reader = Cursor::new(raw_data.as_bytes());

        let target_record = YPBankRecord::new(
            1000000000000000,
            TransactionType::Deposit,
            0,
            9223372036854775807,
            100,
            1633036860000,
            TransactionStatus::Failure,
            "\"Record number 1\"".to_string(),
        );

        let result = YPBankTxtRecordParser::from_read(&mut reader);
        assert!(result.is_ok(), "Parsing should succeed");
        let record_opt = result.expect("Should parse successfully");
        assert!(record_opt.is_some(), "Should return Some(record)");
        assert_eq!(record_opt.expect("Should have a record"), target_record);
    }

    #[test]
    fn test_write_to_regular_case() {
        let record = YPBankRecord::new(
            1000000000000000,
            TransactionType::Deposit,
            0,
            9223372036854775807,
            100,
            1633036860000,
            TransactionStatus::Failure,
            "\"Record number 1\"".to_string(),
        );
        let raw_data = "TX_ID: 1000000000000000\nTX_TYPE: DEPOSIT\nFROM_USER_ID: 0\nTO_USER_ID: 9223372036854775807\nAMOUNT: 100\nTIMESTAMP: 1633036860000\nSTATUS: FAILURE\nDESCRIPTION: \"Record number 1\"\n\n";

        let mut writer = Cursor::new(Vec::new());
        let result = YPBankTxtRecordParser::write_to(&record, &mut writer);
        assert!(result.is_ok(), "Writing should succeed");

        let written =
            String::from_utf8(writer.into_inner()).expect("Written data should be valid UTF-8");
        assert_eq!(written, raw_data);
    }
}

#[cfg(test)]
mod txt_parser_tests {
    use super::*;
    use crate::common::TransactionStatus;
    use std::io::Cursor;

    #[test]
    fn test_from_read_multiple_records() {
        let raw_data = "# Record 1 (DEPOSIT)\nTX_TYPE: DEPOSIT\nTO_USER_ID: 9223372036854775807\nFROM_USER_ID: 0\nTIMESTAMP: 1633036860000\nDESCRIPTION: \"Record number 1\"\nTX_ID: 1000000000000000\nAMOUNT: 100\nSTATUS: FAILURE\n\n# Record 2 (TRANSFER)\nDESCRIPTION: \"Record number 2\"\nTIMESTAMP: 1633036920000\nSTATUS: PENDING\nAMOUNT: 200\nTX_ID: 1000000000000001\nTX_TYPE: TRANSFER\nFROM_USER_ID: 9223372036854775807\nTO_USER_ID: 9223372036854775807\n\n";
        let mut reader = Cursor::new(raw_data.as_bytes());

        let target_records = vec![
            YPBankRecord::new(
                1000000000000000,
                TransactionType::Deposit,
                0,
                9223372036854775807,
                100,
                1633036860000,
                TransactionStatus::Failure,
                "\"Record number 1\"".to_string(),
            ),
            YPBankRecord::new(
                1000000000000001,
                TransactionType::Transfer,
                9223372036854775807,
                9223372036854775807,
                200,
                1633036920000,
                TransactionStatus::Pending,
                "\"Record number 2\"".to_string(),
            ),
        ];

        let result = TxtParser::from_read(&mut reader);
        assert!(result.is_ok(), "Parsing should succeed");
        let records = result.expect("Should parse successfully");
        assert_eq!(records.len(), 2);
        assert_eq!(records, target_records);
    }

    #[test]
    fn test_write_to_multiple_records() {
        let records = vec![
            YPBankRecord::new(
                1000000000000000,
                TransactionType::Deposit,
                0,
                9223372036854775807,
                100,
                1633036860000,
                TransactionStatus::Failure,
                "\"Record number 1\"".to_string(),
            ),
            YPBankRecord::new(
                1000000000000001,
                TransactionType::Transfer,
                9223372036854775807,
                9223372036854775807,
                200,
                1633036920000,
                TransactionStatus::Pending,
                "\"Record number 2\"".to_string(),
            ),
        ];

        let raw_data = "TX_ID: 1000000000000000\nTX_TYPE: DEPOSIT\nFROM_USER_ID: 0\nTO_USER_ID: 9223372036854775807\nAMOUNT: 100\nTIMESTAMP: 1633036860000\nSTATUS: FAILURE\nDESCRIPTION: \"Record number 1\"\nTX_ID: 1000000000000001\nTX_TYPE: TRANSFER\nFROM_USER_ID: 9223372036854775807\nTO_USER_ID: 9223372036854775807\nAMOUNT: 200\nTIMESTAMP: 1633036920000\nSTATUS: PENDING\nDESCRIPTION: \"Record number 2\"\n";

        let mut writer = Cursor::new(Vec::new());
        let result = TxtParser::write_to(&mut writer, &records);
        assert!(result.is_ok(), "Writing should succeed");

        let written =
            String::from_utf8(writer.into_inner()).expect("Written data should be valid UTF-8");
        assert_eq!(written, raw_data);
    }
}
