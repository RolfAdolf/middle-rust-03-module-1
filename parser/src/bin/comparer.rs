use clap::Parser;
use parser::{CommonParser, Format, ParseError};
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file1: String,

    #[arg(long)]
    format1: String,

    #[arg(long)]
    file2: String,

    #[arg(long)]
    format2: String,
}

impl Args {
    fn format1(&self) -> Result<Format, ParseError> {
        Format::from_str(&self.format1)
    }

    fn format2(&self) -> Result<Format, ParseError> {
        Format::from_str(&self.format2)
    }
}

fn run_logic<R: std::io::Read>(file1: &mut R, format1: Format, file2: &mut R, format2: Format) {
    let parser1 = CommonParser::new(format1);
    let parser2 = CommonParser::new(format2);
    let records1 = match parser1.from_read(file1) {
        Ok(records) => records,
        Err(err) => {
            println!("Failed to read first file: {err}");
            return;
        }
    };
    let records2 = match parser2.from_read(file2) {
        Ok(records) => records,
        Err(err) => {
            println!("Failed to read second file: {err}");
            return;
        }
    };

    if records1.len() != records2.len() {
        println!("Files have different number of transactions");
        return;
    }

    for (record1, record2) in records1.iter().zip(records2.iter()) {
        if record1 != record2 {
            println!("Found different transactions");
            println!("Record 1: {:?}", record1);
            println!("Record 2: {:?}", record2);
            return;
        }
    }

    println!("All transactions are identical");
}

fn main() {
    let args = Args::parse();

    let format1 = match args.format1() {
        Ok(format) => format,
        Err(err) => {
            println!("Invalid format for first file: {err}");
            return;
        }
    };
    let format2 = match args.format2() {
        Ok(format) => format,
        Err(err) => {
            println!("Invalid format for second file: {err}");
            return;
        }
    };

    let mut file1 = match std::fs::File::open(&args.file1) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open first file {}: {err}", args.file1);
            return;
        }
    };
    let mut file2 = match std::fs::File::open(&args.file2) {
        Ok(file) => file,
        Err(err) => {
            println!("Failed to open second file {}: {err}", args.file2);
            return;
        }
    };

    run_logic(&mut file1, format1, &mut file2, format2);
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

    #[test]
    fn test_identical_records_same_format() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records = vec![record1, record2];

        let csv_data = create_csv_data(records);
        let mut file1 = Cursor::new(csv_data.clone());
        let mut file2 = Cursor::new(csv_data);

        run_logic(&mut file1, Format::Csv, &mut file2, Format::Csv);
    }

    #[test]
    fn test_different_formats() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let records1 = vec![record1, record2];
        let record3 = create_test_record(1000000000000000, 100);
        let record4 = create_test_record(1000000000000001, 200);
        let records2 = vec![record3, record4];

        let csv_data = create_csv_data(records1);
        let txt_data = create_txt_data(records2);

        let mut file1 = Cursor::new(csv_data);
        let mut file2 = Cursor::new(txt_data);

        run_logic(&mut file1, Format::Csv, &mut file2, Format::Txt);
    }

    #[test]
    fn test_different_number_of_records() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);
        let record3 = create_test_record(1000000000000002, 300);

        let csv_data1 = create_csv_data(vec![record1, record2]);
        let csv_data2 = create_csv_data(vec![record3]);

        let mut file1 = Cursor::new(csv_data1);
        let mut file2 = Cursor::new(csv_data2);

        run_logic(&mut file1, Format::Csv, &mut file2, Format::Csv);
    }

    #[test]
    fn test_different_records() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);

        let record3 = create_test_record(1000000000000000, 100);
        let record4 = YPBankRecord::new(
            1000000000000001,
            TransactionType::Deposit,
            0,
            9223372036854775807,
            300, // != 200 (record2 amount)
            1633036860000,
            TransactionStatus::Success,
            "\"Record number 1000000000000001\"".to_string(),
        );

        let csv_data1 = create_csv_data(vec![record1, record2]);
        let csv_data2 = create_csv_data(vec![record3, record4]);

        let mut file1 = Cursor::new(csv_data1);
        let mut file2 = Cursor::new(csv_data2);

        run_logic(&mut file1, Format::Csv, &mut file2, Format::Csv);
    }

    #[test]
    fn test_empty_files() {
        let csv_data =
            b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n".to_vec();

        let mut file1 = Cursor::new(csv_data.clone());
        let mut file2 = Cursor::new(csv_data);

        run_logic(&mut file1, Format::Csv, &mut file2, Format::Csv);
    }

    #[test]
    fn test_all_formats() {
        let record1 = create_test_record(1000000000000000, 100);
        let record2 = create_test_record(1000000000000001, 200);

        let record3 = create_test_record(1000000000000000, 100);
        let record4 = create_test_record(1000000000000001, 200);

        let record5 = create_test_record(1000000000000000, 100);
        let record6 = create_test_record(1000000000000001, 200);

        let csv_data = create_csv_data(vec![record1, record2]);
        let txt_data = create_txt_data(vec![record3, record4]);
        let bin_data = create_bin_data(vec![record5, record6]);

        // CSV and TXT
        let mut file1 = Cursor::new(csv_data.clone());
        let mut file2 = Cursor::new(txt_data.clone());
        run_logic(&mut file1, Format::Csv, &mut file2, Format::Txt);

        // TXT and BIN
        let mut file1 = Cursor::new(txt_data);
        let mut file2 = Cursor::new(bin_data.clone());
        run_logic(&mut file1, Format::Txt, &mut file2, Format::Bin);

        // CSV and BIN
        let mut file1 = Cursor::new(csv_data);
        let mut file2 = Cursor::new(bin_data);
        run_logic(&mut file1, Format::Csv, &mut file2, Format::Bin);
    }
}
