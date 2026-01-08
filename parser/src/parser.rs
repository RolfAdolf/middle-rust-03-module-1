use crate::error::ParseError;
use crate::record::YPBankRecord;

pub trait YPBankRecordParser {
    fn from_read<R: std::io::BufRead>(r: &mut R) -> Result<Option<YPBankRecord>, ParseError>;
    fn write_to<W: std::io::Write>(record: &YPBankRecord, w: &mut W) -> Result<(), ParseError>;
}

pub trait Parser<RecordParser: YPBankRecordParser> {
    fn from_read<Reader: std::io::Read>(r: &mut Reader) -> Result<Vec<YPBankRecord>, ParseError> {
        let mut buf_reader = std::io::BufReader::new(r);

        Self::pre_read(&mut buf_reader)?;

        let mut records: Vec<YPBankRecord> = vec![];
        loop {
            let record_opt = RecordParser::from_read(&mut buf_reader)?;
            match record_opt {
                Some(record) => records.push(record),
                None => break,
            }
        }

        Ok(records)
    }

    fn write_to<Writer: std::io::Write>(
        w: &mut Writer,
        records: &Vec<YPBankRecord>,
    ) -> Result<(), ParseError> {
        Self::pre_write(w)?;

        for record in records {
            RecordParser::write_to(record, w)?;
        }

        Ok(())
    }

    fn pre_read<Reader: std::io::BufRead>(_: &mut Reader) -> Result<(), ParseError> {
        Ok(())
    }

    fn pre_write<Writer: std::io::Write>(_: &mut Writer) -> Result<(), ParseError> {
        Ok(())
    }
}
