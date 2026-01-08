use crate::common::{
    TransactionStatus, TransactionType, read_i64_from_bytes, read_u8_from_bytes,
    read_u32_from_bytes, read_u64_from_bytes, validate_from_user_id, validate_to_user_id,
};
use crate::error::ParseError;
use crate::parser::{Parser, YPBankRecordParser};
use crate::record::YPBankRecord;

pub struct YPBankBinRecordParser {}

impl YPBankBinRecordParser {
    const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

    fn validate_magic<R: std::io::BufRead>(r: &mut R) -> Result<(), ParseError> {
        let mut magic = [0; 4];
        if let Err(err) = r.read_exact(&mut magic) {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                return Err(ParseError::UnexpectedEOF);
            }

            return Err(ParseError::IOError(err.to_string()));
        }

        if magic != Self::MAGIC {
            let magic_str = magic
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>()
                .join(" ");
            return Err(ParseError::InvalidMagic(magic_str));
        }

        Ok(())
    }

    fn parse_record_size<R: std::io::BufRead>(r: &mut R) -> Result<u32, ParseError> {
        read_u32_from_bytes(r)
    }

    fn parse_record<R: std::io::BufRead>(r: &mut R) -> Result<YPBankRecord, ParseError> {
        let id = read_u64_from_bytes(r)?;
        let transaction_type = TransactionType::from_int(read_u8_from_bytes(r)?)?;
        let from_user_id = validate_from_user_id(read_u64_from_bytes(r)?, transaction_type)?;
        let to_user_id = validate_to_user_id(read_u64_from_bytes(r)?, transaction_type)?;
        let amount = read_i64_from_bytes(r)?;
        let ts = read_u64_from_bytes(r)?;
        let status = TransactionStatus::from_int(read_u8_from_bytes(r)?)?;
        let description = Self::read_description_from_bytes(r)?;

        Ok(YPBankRecord::new(
            id,
            transaction_type,
            from_user_id,
            to_user_id,
            amount,
            ts,
            status,
            description,
        ))
    }

    fn read_description_from_bytes<R: std::io::BufRead>(r: &mut R) -> Result<String, ParseError> {
        let desc_len = read_u32_from_bytes(r)? as usize;

        let mut desc_bytes = vec![0; desc_len];
        r.read_exact(&mut desc_bytes)?;

        String::from_utf8(desc_bytes).map_err(|err| ParseError::InvalidRawValue(err.to_string()))
    }

    fn get_record_size(description: &str) -> u32 {
        8 + 1 + 8 + 8 + 8 + 8 + 1 + 4 + description.len() as u32
    }
}

impl YPBankRecordParser for YPBankBinRecordParser {
    fn from_read<R: std::io::BufRead>(r: &mut R) -> Result<Option<YPBankRecord>, ParseError> {
        if let Err(err) = Self::validate_magic(r) {
            if err == ParseError::UnexpectedEOF {
                return Ok(None);
            }

            return Err(err);
        }

        let record_size = Self::parse_record_size(r)?;
        if record_size == 0 {
            return Ok(None);
        }

        let record = Self::parse_record(r)?;
        Ok(Some(record))
    }

    fn write_to<W: std::io::Write>(record: &YPBankRecord, w: &mut W) -> Result<(), ParseError> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&Self::MAGIC);
        bytes.extend_from_slice(&Self::get_record_size(&record.description).to_be_bytes());

        bytes.extend_from_slice(&record.id.to_be_bytes());
        bytes.extend_from_slice(&record.transaction_type.as_int().to_be_bytes());
        bytes.extend_from_slice(&record.from_user_id.to_be_bytes());
        bytes.extend_from_slice(&record.to_user_id.to_be_bytes());
        bytes.extend_from_slice(&record.amount.to_be_bytes());
        bytes.extend_from_slice(&record.ts.to_be_bytes());
        bytes.extend_from_slice(&record.status.as_int().to_be_bytes());
        bytes.extend_from_slice(&(record.description.len() as u32).to_be_bytes());
        bytes.extend_from_slice(record.description.as_bytes());

        w.write_all(&bytes)?;

        Ok(())
    }
}

pub struct BinParser {}

impl Parser<YPBankBinRecordParser> for BinParser {}

#[cfg(test)]
mod yp_bank_bin_record_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_from_read_regular_case() {
        let description = "\"Record number 1\"";
        let desc_bytes = description.as_bytes();
        let desc_len = desc_bytes.len() as u32;

        let mut data = Vec::new();

        let record_size: u32 = 46 + desc_len;

        data.extend_from_slice(&[0x59, 0x50, 0x42, 0x4E]); // MAGIC
        data.extend_from_slice(&record_size.to_be_bytes()); // RECORD_SIZE
        data.extend_from_slice(&1000000000000000u64.to_be_bytes()); // TX_ID
        data.push(TransactionType::Deposit.as_int()); // TX_TYPE
        data.extend_from_slice(&0u64.to_be_bytes()); // FROM_USER_ID
        data.extend_from_slice(&9223372036854775807u64.to_be_bytes()); // TO_USER_ID
        data.extend_from_slice(&100i64.to_be_bytes()); // AMOUNT
        data.extend_from_slice(&1633036860000u64.to_be_bytes()); // TIMESTAMP
        data.push(TransactionStatus::Failure.as_int()); // STATUS
        data.extend_from_slice(&desc_len.to_be_bytes()); // DESC_LEN
        data.extend_from_slice(desc_bytes); // DESCRIPTION

        let mut reader = Cursor::new(data);
        let result = YPBankBinRecordParser::from_read(&mut reader);

        let target_record = YPBankRecord::new(
            1000000000000000,
            TransactionType::Deposit,
            0,
            9223372036854775807,
            100,
            1633036860000,
            TransactionStatus::Failure,
            description.to_string(),
        );

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

        let mut writer = Cursor::new(Vec::new());
        let result = YPBankBinRecordParser::write_to(&record, &mut writer);
        assert!(result.is_ok(), "Writing should succeed");

        let written = writer.into_inner();

        assert_eq!(&written[0..4], &[0x59, 0x50, 0x42, 0x4E]);

        let record_size = u32::from_be_bytes([written[4], written[5], written[6], written[7]]);
        assert_eq!(record_size, 46 + record.description.len() as u32);

        let mut reader = Cursor::new(&written);
        let read_result = YPBankBinRecordParser::from_read(&mut reader);
        assert!(read_result.is_ok(), "Reading should succeed");
        let read_record_opt = read_result.expect("Should parse successfully");
        let read_record = read_record_opt.expect("Should have a record");
        assert_eq!(read_record, record);
    }

    #[test]
    fn test_from_read_invalid_magic() {
        let mut data = Vec::new();

        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        data.extend_from_slice(&0u32.to_be_bytes());

        let mut reader = Cursor::new(data);
        let result = YPBankBinRecordParser::from_read(&mut reader);

        assert!(result.is_err(), "Should return an error");

        let error = result.err().expect("Should return an error");
        assert!(matches!(error, ParseError::InvalidMagic(_)));
    }

    #[test]
    fn test_from_read_eof() {
        let mut reader = Cursor::new(Vec::<u8>::new());
        let result = YPBankBinRecordParser::from_read(&mut reader);

        assert!(result.is_ok(), "EOF should return Ok(None)");
        assert!(
            result.expect("Should parse successfully").is_none(),
            "Should return None on EOF"
        );
    }
}

#[cfg(test)]
mod bin_parser_tests {
    use super::*;
    use std::io::Cursor;

    fn create_record_data(
        id: u64,
        tx_type: u8,
        from: u64,
        to: u64,
        amount: i64,
        ts: u64,
        status: u8,
        desc: &str,
    ) -> Vec<u8> {
        let desc_bytes = desc.as_bytes();
        let desc_len = desc_bytes.len() as u32;
        let record_size: u32 = 46 + desc_len;

        let mut data = Vec::new();
        data.extend_from_slice(&[0x59, 0x50, 0x42, 0x4E]);
        data.extend_from_slice(&record_size.to_be_bytes());
        data.extend_from_slice(&id.to_be_bytes());
        data.push(tx_type);
        data.extend_from_slice(&from.to_be_bytes());
        data.extend_from_slice(&to.to_be_bytes());
        data.extend_from_slice(&amount.to_be_bytes());
        data.extend_from_slice(&ts.to_be_bytes());
        data.push(status);
        data.extend_from_slice(&desc_len.to_be_bytes());
        data.extend_from_slice(desc_bytes);
        data
    }

    #[test]
    fn test_from_read_multiple_records() {
        let mut data = Vec::new();
        data.extend_from_slice(&create_record_data(
            1000000000000000,
            0,
            0,
            9223372036854775807,
            100,
            1633036860000,
            1,
            "\"Record number 1\"",
        ));

        data.extend_from_slice(&create_record_data(
            1000000000000001,
            1,
            9223372036854775807,
            9223372036854775807,
            200,
            1633036920000,
            2,
            "\"Record number 2\"",
        ));

        let mut reader = Cursor::new(data);
        let result = BinParser::from_read(&mut reader);

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

        let mut writer = Cursor::new(Vec::new());
        let result = BinParser::write_to(&mut writer, &records);
        assert!(result.is_ok(), "Writing should succeed");

        let written = writer.into_inner();

        let mut reader = Cursor::new(&written);
        let read_result = BinParser::from_read(&mut reader);
        assert!(read_result.is_ok(), "Reading should succeed");
        let read_records = read_result.expect("Should parse successfully");
        assert_eq!(read_records.len(), 2);
        assert_eq!(read_records, records);
    }
}
