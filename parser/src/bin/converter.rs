use clap::Parser;
use parser::{CommonParser, Format, ParseError};
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    input: String,

    #[arg(long)]
    input_format: String,

    #[arg(long)]
    output_format: String,
}

impl Args {
    fn input_format(&self) -> Result<Format, ParseError> {
        Format::from_str(&self.input_format)
    }

    fn output_format(&self) -> Result<Format, ParseError> {
        Format::from_str(&self.output_format)
    }
}

fn run_logic<R: std::io::Read, W: std::io::Write>(
    input_file: &mut R,
    input_format: Format,
    output_format: Format,
    output_file: &mut W,
) {
    let input_parser = CommonParser::new(input_format);
    let output_parser = CommonParser::new(output_format);
    let records = match input_parser.from_read(input_file) {
        Ok(records) => records,
        Err(err) => {
            println!("Failed to read input: {err}");
            return;
        }
    };
    if let Err(err) = output_parser.write_to(output_file, &records) {
        println!("Failed to write output: {err}");
    }
}

fn main() {
    let args = Args::parse();

    let mut input_file = match std::fs::File::open(&args.input) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open input file {}: {err}", args.input);
            return;
        }
    };
    let mut output_file = std::io::stdout();

    let input_format = match args.input_format() {
        Ok(format) => format,
        Err(err) => {
            println!("Invalid input format {}: {err}", args.input_format);
            return;
        }
    };

    let output_format = match args.output_format() {
        Ok(format) => format,
        Err(err) => {
            println!("Invalid output format {}: {err}", args.output_format);
            return;
        }
    };

    run_logic(
        &mut input_file,
        input_format,
        output_format,
        &mut output_file,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::{TransactionStatus, TransactionType, YPBankRecord};
    use std::io::Cursor;

    fn create_csv_data(records: Vec<YPBankRecord>) -> Vec<u8> {
        let mut data =
            b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n".to_vec();
        for record in records {
            let line = format!(
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
            data.extend_from_slice(line.as_bytes());
        }
        data
    }

    fn create_txt_data(records: Vec<YPBankRecord>) -> Vec<u8> {
        let mut data = Vec::new();
        for record in records {
            let record_str = format!(
                "TX_ID: {}\nTX_TYPE: {}\nFROM_USER_ID: {}\nTO_USER_ID: {}\nAMOUNT: {}\nTIMESTAMP: {}\nSTATUS: {}\nDESCRIPTION: {}\n",
                record.id,
                record.transaction_type.as_str(),
                record.from_user_id,
                record.to_user_id,
                record.amount,
                record.ts,
                record.status.as_str(),
                record.description
            );
            data.extend_from_slice(record_str.as_bytes());
        }
        data
    }

    fn create_bin_data(records: Vec<YPBankRecord>) -> Vec<u8> {
        let mut data = Vec::new();
        for record in records {
            let desc_len = record.description.len() as u32;
            let record_size: u32 = 46 + desc_len;

            data.extend_from_slice(&[0x59, 0x50, 0x42, 0x4E]);
            data.extend_from_slice(&record_size.to_be_bytes());
            data.extend_from_slice(&record.id.to_be_bytes());
            data.push(record.transaction_type.as_int());
            data.extend_from_slice(&record.from_user_id.to_be_bytes());
            data.extend_from_slice(&record.to_user_id.to_be_bytes());
            data.extend_from_slice(&record.amount.to_be_bytes());
            data.extend_from_slice(&record.ts.to_be_bytes());
            data.push(record.status.as_int());
            data.extend_from_slice(&desc_len.to_be_bytes());
            data.extend_from_slice(&record.description.as_bytes());
        }
        data
    }

    fn create_test_record(id: u64, amount: i64) -> YPBankRecord {
        YPBankRecord::new(
            id,
            TransactionType::Deposit,
            0,
            9223372036854775807,
            amount,
            1633036860000,
            TransactionStatus::Success,
            format!("\"Record number {}\"", id),
        )
    }

    fn parse_output_csv(output: &[u8]) -> Vec<YPBankRecord> {
        let mut reader = Cursor::new(output);
        let parser = CommonParser::new(Format::Csv);
        parser
            .from_read(&mut reader)
            .expect("Should parse output as CSV")
    }

    fn parse_output_txt(output: &[u8]) -> Vec<YPBankRecord> {
        let mut reader = Cursor::new(output);
        let parser = CommonParser::new(Format::Txt);
        parser
            .from_read(&mut reader)
            .expect("Should parse output as TXT")
    }

    fn parse_output_bin(output: &[u8]) -> Vec<YPBankRecord> {
        let mut reader = Cursor::new(output);
        let parser = CommonParser::new(Format::Bin);
        parser
            .from_read(&mut reader)
            .expect("Should parse output as BIN")
    }

    #[test]
    fn test_csv_to_txt() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_csv_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Csv, Format::Txt, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_txt(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_csv_to_bin() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_csv_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Csv, Format::Bin, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_bin(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_txt_to_csv() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_txt_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Txt, Format::Csv, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_csv(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_txt_to_bin() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_txt_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Txt, Format::Bin, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_bin(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_bin_to_csv() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_bin_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Bin, Format::Csv, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_csv(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_bin_to_txt() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_bin_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Bin, Format::Txt, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_txt(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_same_format() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let input_data = create_csv_data(records);
        let mut input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Csv, Format::Csv, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_csv(&output_data);
        let expected = vec![
            create_test_record(1000000000000000, 100),
            create_test_record(1000000000000001, 200),
        ];
        assert_eq!(parsed_records, expected);
    }

    #[test]
    fn test_empty_file() {
        let csv_data =
            b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n".to_vec();

        let mut input = Cursor::new(csv_data);
        let mut output = Cursor::new(Vec::new());

        run_logic(&mut input, Format::Csv, Format::Txt, &mut output);

        let output_data = output.into_inner();
        let parsed_records = parse_output_txt(&output_data);
        assert_eq!(parsed_records.len(), 0);
    }
}
