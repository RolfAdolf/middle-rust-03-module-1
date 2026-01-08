# Bank Transaction Record Parser

A Rust project for parsing, converting, and comparing bank transaction records in multiple formats.

## Project Structure

```
middle-rust-03-module-1/
├── parser/              # Core parsing library
│   ├── src/
│   │   ├── bin/         # Command-line tools
│   │   │   ├── comparer.rs
│   │   │   └── converter.rs
│   │   └── ...          # Library modules
│   ├── examples/        # Example data files
│   └── README.md        # Parser library documentation
└── README.md            # This file
```

## Features

- **Multi-format Support**: Read and write bank transaction records in CSV, TXT, and binary formats
- **Format Conversion**: Convert between any supported formats
- **Record Comparison**: Compare two record files regardless of format
- **Type Safety**: Strongly typed API with comprehensive error handling
- **CLI Tools**: Command-line utilities for common operations

## Quick Start

### Installation

```bash
cd parser
cargo build
```

### Using the Converter

Convert records between formats:

```bash
# CSV to TXT
cargo run --bin converter -- --input examples/records_example.csv --input-format csv --output-format txt > output.txt

# Binary to CSV
cargo run --bin converter -- --input examples/records_example.bin --input-format binary --output-format csv
```

### Using the Comparer

Compare two record files:

```bash
# Compare two files
cargo run --bin comparer -- --file1 file1.csv --format1 csv --file2 file2.txt --format2 txt
```

## Supported Formats

### CSV Format
- Comma-separated values with header
- Fields: TX_ID, TX_TYPE, FROM_USER_ID, TO_USER_ID, AMOUNT, TIMESTAMP, STATUS, DESCRIPTION
- Supports quoted fields

### TXT Format
- Key-value pairs (one per line)
- Format: `FIELD_NAME: value`
- Supports comments (lines starting with `#`)
- Fields can appear in any order

### Binary Format
- Fixed-size binary format
- Magic bytes: `YPBN` (0x59 0x50 0x42 0x4E)
- Efficient for large datasets

## Library Usage

The parser library can be used in your own Rust projects:

```rust
use parser::{CommonParser, Format};
use std::fs::File;

// Read records
let parser = CommonParser::new(Format::Csv);
let mut file = File::open("records.csv")?;
let records = parser.from_read(&mut file)?;

// Write records
let output_parser = CommonParser::new(Format::Txt);
let mut output = File::create("output.txt")?;
output_parser.write_to(&mut output, &records)?;
```

## Documentation

- [Parser Library Documentation](parser/README.md) - Detailed library API documentation
- [CLI Tools Documentation](parser/src/bin/README.md) - Command-line tool usage

## Building

```bash
# Build everything
cargo build

# Build specific binary
cargo build --bin converter
cargo build --bin comparer

# Run tests
cargo test
```

## Examples

Example files are provided in `parser/examples/`:
- `records_example.csv` - CSV format example
- `records_example.txt` - TXT format example
- `records_example.bin` - Binary format example
