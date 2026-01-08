# Parser Library

A Rust library for parsing and writing bank transaction records in multiple formats: CSV, TXT, and binary.

## Features

- **Multi-format support**: Read and write records in CSV, TXT, and binary formats
- **Type-safe parsing**: Strongly typed error handling with `ParseError`
- **Unified interface**: `CommonParser` provides a single API for all formats
- **Validation**: Automatic validation of transaction types and user IDs

## Supported Formats

### CSV Format
- Comma-separated values with header row
- Fields: TX_ID, TX_TYPE, FROM_USER_ID, TO_USER_ID, AMOUNT, TIMESTAMP, STATUS, DESCRIPTION
- Supports quoted fields with commas

### TXT Format
- Key-value pairs separated by colons
- One field per line
- Supports comments (lines starting with `#`)
- Fields can appear in any order

### Binary Format
- Fixed-size binary format with magic bytes (`YPBN`)
- Efficient for large datasets
- Each record includes size information

## Usage

### Basic Example

```rust
use parser::{CommonParser, Format};
use std::fs::File;

// Read records from a CSV file
let parser = CommonParser::new(Format::Csv);
let mut file = File::open("records.csv")?;
let records = parser.from_read(&mut file)?;

// Write records to a TXT file
let output_parser = CommonParser::new(Format::Txt);
let mut output_file = File::create("output.txt")?;
output_parser.write_to(&mut output_file, &records)?;
```

### Working with Records

```rust
use parser::{YPBankRecord, TransactionType, TransactionStatus};

let record = YPBankRecord::new(
    1000000000000000,
    TransactionType::Deposit,
    0,
    9223372036854775807,
    100,
    1633036860000,
    TransactionStatus::Success,
    "Transaction description".to_string(),
);
```

## Examples

Example files are available in the `examples/` directory:
- `records_example.csv` - CSV format example
- `records_example.txt` - TXT format example
- `records_example.bin` - Binary format example

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Documentation

Generate documentation with:

```bash
cargo doc --open
```

