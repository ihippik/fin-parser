# Fin-parser

A command-line tool for conversion between financial data formats such as 
CSV, MT940, CAMT.053, and XML.

🚀 Features

🔄 Convert between multiple financial formats:

CSV — simple tabular data

MT940 — SWIFT statement format

CAMT.053 — ISO 20022 XML bank statement

XML — simplified internal XML representation

🧩 Works with both files and standard input/output

parser --in-format mt940 --out-format camt053 --input input.mt940 --output output.xml

## 💡 Usage
`parser --in-format mt940 --out-format camt053 --input input.mt940 --output output.xml
`

## ⚙️ Options

| Option                  | Description                                                                    | Example                      |
|:------------------------|:-------------------------------------------------------------------------------|:-----------------------------|
| `--input <PATH>`        | Input file (optional, defaults to **stdin**)                                   | `--input transactions.mt940` |
| `--output <PATH>`       | Output file (optional, defaults to **stdout**)                                 | `--output result.xml`        |
| `--in-format <FORMAT>`  | Input format (required). Possible values:<br>`csv`, `mt940`, `camt053`, `xml`  | `--in-format mt940`          |
| `--out-format <FORMAT>` | Output format (required). Possible values:<br>`csv`, `mt940`, `camt053`, `xml` | `--out-format camt053`       |
| `-h, --help`            | Show help information                                                          | `parser --help`              |
| `-V, --version`         | Show version information                                                       | `parser --version`           |

## 🧠 Examples

### Convert MT940 → CAMT.053

```bash
parser --in-format mt940 --out-format camt053 \
       --input bank.mt940 --output statement.xml
```

### Convert CSV → XML (output to stdout)
```bash
parser --in-format csv --out-format xml --input data.csv
```

### Convert CSV → XML (using stdin and stdout)
```bash
cat data.csv | parser --in-format csv --out-format xml > result.xml
```

### Convert CAMT.053 → CSV
```bash
parser --in-format camt053 --out-format csv \
       --input statement.xml --output report.csv
```
