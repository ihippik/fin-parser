use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("parser").expect("binary 'parser' not found")
}

#[test]
fn test_shows_help() {
    let mut cmd = bin();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("USAGE")));
}

#[test]
fn shows_version() {
    let mut cmd = bin();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn mt940_to_camt053_smoke() {
    let mt940 = r#"
:20:STATEMENT1
:25:DE0012345678
:60F:C251001EUR1000,00
:61:2510011001C100,00NTRFNONREF
:86:Salary October
:62F:C251031EUR1100,00
"#;

    let dir = tempdir().unwrap();
    let input = dir.path().join("input.mt940");
    let output = dir.path().join("out.xml");

    fs::write(&input, mt940).unwrap();

    let mut cmd = bin();
    cmd.args([
        "--in-format",
        "mt940",
        "--out-format",
        "camt053",
        "--input",
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);

    cmd.assert().success();

    let xml = fs::read_to_string(&output).unwrap();
    assert!(xml.contains("<Document"));
    assert!(xml.contains("BkToCstmrStmt")); // корневой блок camt.053
    assert!(xml.contains("<Stmt>"));
    assert!(xml.contains("<Ntry>"));
}

#[test]
fn mt940_to_xml_smoke_stdout() {
    let mt940 = r#"
:20:STATEMENT1
:25:DE0012345678
:60F:C251001EUR1000,00
:61:2510011001C100,00NTRFNONREF
:86:Salary October
:62F:C251031EUR1100,00
"#;

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("input.mt940");
    fs::write(&input_path, mt940).unwrap();

    let mut cmd = bin();
    cmd.args([
        "--in-format", "mt940",
        "--out-format", "xml",
        "--input", input_path.to_str().unwrap(),
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("<").and(predicate::str::contains(">")))
        .stdout(predicate::str::contains("XmlStatement").or(predicate::str::contains("Document")));
}

#[test]
fn camt053_to_csv_smoke() {
    let camt = r#"
<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
  <BkToCstmrStmt>
    <Stmt>
      <Id>STATEMENT1</Id>
      <Acct><Id><IBAN>DE0012345678</IBAN></Id></Acct>
      <Bal>
        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="EUR">1000.00</Amt>
        <Dt><Dt>2025-10-01</Dt></Dt>
      </Bal>
      <Ntry>
        <Amt Ccy="EUR">100.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <ValDt><Dt>2025-10-01</Dt></ValDt>
        <BookgDt><Dt>2025-10-01</Dt></BookgDt>
        <AddtlNtryInf>Salary October</AddtlNtryInf>
      </Ntry>
      <Bal>
        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
        <Amt Ccy="EUR">1100.00</Amt>
        <Dt><Dt>2025-10-31</Dt></Dt>
      </Bal>
    </Stmt>
  </BkToCstmrStmt>
</Document>
"#;

    let dir = tempdir().unwrap();
    let input = dir.path().join("statement.xml");
    let output = dir.path().join("out.csv");
    fs::write(&input, camt).unwrap();

    let mut cmd = bin();
    cmd.args([
        "--in-format",
        "camt053",
        "--out-format",
        "csv",
        "--input",
        input.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
    ]);

    cmd.assert().success();

    let csv = fs::read_to_string(&output).unwrap();
    assert!(csv.contains("Salary October"));
    assert!(csv.contains("100"));
    assert!(csv.contains("2025-10-01"));
}
