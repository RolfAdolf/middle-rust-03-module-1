# Command-Line Tools

This directory contains command-line binaries for working with bank transaction records.

## Available Tools

### `converter`

Converts bank transaction records between different formats (CSV, TXT, binary).

#### Usage

```bash
cargo run --bin converter -- --input <INPUT_FILE> --input-format <FORMAT> --output-format <FORMAT>
```

#### Arguments

- `--input <INPUT_FILE>`: Path to the input file
- `--input-format <FORMAT>`: Format of the input file (`csv`, `txt`, or `binary`)
- `--output-format <FORMAT>`: Desired output format (`csv`, `txt`, or `binary`)

#### Examples

```bash
# Convert CSV to TXT
cargo run --bin converter -- --input examples/records_example.csv --input-format csv --output-format txt > output.txt

# Convert binary to CSV
cargo run --bin converter -- --input examples/records_example.bin --input-format binary --output-format csv

# Convert TXT to binary (output to stdout)
cargo run --bin converter -- --input examples/records_example.txt --input-format txt --output-format binary
```

#### Output

The converter writes the converted records to stdout. You can redirect to a file:

```bash
cargo run --bin converter -- --input input.csv --input-format csv --output-format txt > output.txt
```

### `comparer`

Compares two bank transaction record files and reports differences.

#### Usage

```bash
cargo run --bin comparer -- --file1 <FILE1> --format1 <FORMAT> --file2 <FILE2> --format2 <FORMAT>
```

#### Arguments

- `--file1 <FILE1>`: Path to the first file
- `--format1 <FORMAT>`: Format of the first file (`csv`, `txt`, or `binary`)
- `--file2 <FILE2>`: Path to the second file
- `--format2 <FORMAT>`: Format of the second file (`csv`, `txt`, or `binary`)

#### Examples

```bash
# Compare two CSV files
cargo run --bin comparer -- --file1 file1.csv --format1 csv --file2 file2.csv --format2 csv

# Compare CSV and TXT files
cargo run --bin comparer -- --file1 records.csv --format1 csv --file2 records.txt --format2 txt

# Compare binary files
cargo run --bin comparer -- --file1 file1.bin --format1 binary --file2 file2.bin --format2 binary
```

#### Output

The comparer prints one of the following messages:
- `"All transactions are identical"` - Files contain the same records
- `"Files have different number of transactions"` - Files have different record counts
- `"Found different transactions"` - Files have the same count but different records (prints the differing records)

## Format Values

All format arguments accept one of:
- `csv` - CSV format
- `txt` - TXT format
- `binary` - Binary format

## Building Binaries

Build individual binaries:

```bash
# Build converter
cargo build --bin converter

# Build comparer
cargo build --bin comparer

# Build both
cargo build --bin converter --bin comparer
```

## Running Built Binaries

After building, you can run the binaries directly:

```bash
# Run converter
./target/debug/converter --input file.csv --input-format csv --output-format txt

# Run comparer
./target/debug/comparer --file1 file1.csv --format1 csv --file2 file2.csv --format2 csv
```

