use std::io::{BufRead, Write};
use crate::adapter::adapter::{Adapter, Statement};
use crate::adapter::errors::{map_parse_err, AdapterError};
use quick_xml::{Reader,Writer};
use quick_xml::events::{Event,BytesDecl,BytesStart,BytesText};
use quick_xml::escape::unescape;
use crate::adapter::statement::{Balance, DebitCredit, Entry};

pub struct CAMT;

impl Adapter for CAMT {
    fn read_from<R: BufRead>(r: R) -> Result<Statement, AdapterError> {
        let mut reader = Reader::from_reader(r);
        reader.config_mut().trim_text(true);

        let mut st = Statement {
            id: String::new(),
            account_id: String::new(),
            opening_balance: None,
            closing_balance: None,
            entries: Vec::new(),
        };

        // Текущее состояние курсора
        #[derive(Default)]
        struct State {
            in_iban: bool,
            in_stmt_id: bool, // Id именно внутри Stmt (не Acct/Id)
            in_amt: bool,
            amt_ccy: String,
            in_cdt_dbt: bool,
            in_book_dt: bool,
            in_val_dt: bool,
            in_addtl: bool,
            in_ntry_ref: bool,
            pending: Option<Entry>,
        }
        let mut s = State::default();

        let mut buf = Vec::new();

        // Нужен ли нам этот <Id> как идентификатор выписки?
        // Считаем, что он встречается внутри <Stmt> (а не в <Acct><Id>).
        // Мы различаем через флаг in_stmt_id, который поднимаем/опускаем в Start/End.
        // Внешний код устанавливает его там, где нужно.
        // --------------------------------

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.local_name().as_ref() {
                        b"IBAN" => s.in_iban = true,
                        b"Stmt" => {}
                        b"Id" => s.in_stmt_id = true,
                        b"Amt" => {
                            s.in_amt = true;
                            s.amt_ccy = attr_value(&e, b"Ccy").unwrap_or_default();
                        }
                        b"CdtDbtInd" => s.in_cdt_dbt = true,
                        b"BookgDt" => s.in_book_dt = true,
                        b"ValDt" => s.in_val_dt = true,
                        b"AddtlNtryInf" => s.in_addtl = true,
                        b"NtryRef" => s.in_ntry_ref = true,
                        b"Ntry" => {
                            s.pending = Some(Entry {
                                booking_date: String::new(),
                                value_date: String::new(),
                                amount: String::new(),
                                currency: String::from("XXX"),
                                kind: DebitCredit::Credit,
                                description: String::new(),
                                reference: None,
                            });
                        }
                        _ => {}
                    }
                }

                Ok(Event::Text(t)) => {
                    let txt = read_text(t)?;
                    if s.in_iban {
                        st.account_id = txt;
                    } else if s.in_stmt_id && st.id.is_empty() {
                        st.id = txt;
                    } else if s.in_amt {
                        if let Some(ref mut e) = s.pending {
                            e.amount = txt;
                            if !s.amt_ccy.is_empty() {
                                e.currency = s.amt_ccy.clone();
                            }
                        }
                    } else if s.in_cdt_dbt {
                        if let Some(ref mut e) = s.pending {
                            e.kind = match txt.as_str() {
                                "CRDT" => DebitCredit::Credit,
                                "DBIT" => DebitCredit::Debit,
                                other => {
                                    return Err(AdapterError::ParseError(
                                        format!("Unexpected CdtDbtInd `{other}`"),
                                    ))
                                }
                            };
                        }
                    } else if s.in_book_dt {
                        if let Some(ref mut e) = s.pending {
                            e.booking_date = txt;
                        }
                    } else if s.in_val_dt {
                        if let Some(ref mut e) = s.pending {
                            e.value_date = txt;
                        }
                    } else if s.in_addtl {
                        if let Some(ref mut e) = s.pending {
                            e.description = txt;
                        }
                    } else if s.in_ntry_ref {
                        if let Some(ref mut e) = s.pending {
                            e.reference = if txt.is_empty() { None } else { Some(txt) };
                        }
                    }
                }

                Ok(Event::End(e)) => {
                    match e.local_name().as_ref() {
                        b"IBAN" => s.in_iban = false,
                        b"Id" => s.in_stmt_id = false,
                        b"Amt" => s.in_amt = false,
                        b"CdtDbtInd" => s.in_cdt_dbt = false,
                        b"BookgDt" => s.in_book_dt = false,
                        b"ValDt" => s.in_val_dt = false,
                        b"AddtlNtryInf" => s.in_addtl = false,
                        b"NtryRef" => s.in_ntry_ref = false,
                        b"Ntry" => {
                            if let Some(e) = s.pending.take() {
                                st.entries.push(e);
                            }
                        }
                        _ => {}
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => return Err(AdapterError::ParseError(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        if st.id.is_empty() {
            st.id = "none".to_string();
        }

        Ok(st)
    }

    fn write_to<W: Write>(mut writer: W, st: &Statement) -> Result<(), AdapterError> {
        let mut wr = Writer::new_with_indent(&mut writer, b' ', 2);
        wr.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(map_parse_err)?;

        // <Document xmlns="...">
        let mut doc = BytesStart::new("Document");
        doc.push_attribute(("xmlns", "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02"));
        wr.write_event(Event::Start(doc)).map_err(map_parse_err)?;

        // <BkToCstmrStmt><Stmt>
        start(&mut wr, "BkToCstmrStmt")?;
        start(&mut wr, "Stmt")?;

        // <Id>...</Id>
        elem_text(&mut wr, "Id", &st.id)?;

        // <Acct><Id><IBAN>...</IBAN></Id></Acct>
        start(&mut wr, "Acct")?;
        start(&mut wr, "Id")?;
        elem_text(&mut wr, "IBAN", &st.account_id)?;
        end(&mut wr, "Id")?;
        end(&mut wr, "Acct")?;

        // Balances
        if let Some(b) = &st.opening_balance {
            write_balance(&mut wr, "OPBD", b).map_err(map_parse_err)?
        }
        if let Some(b) = &st.closing_balance {
            write_balance(&mut wr, "CLBD", b).map_err(map_parse_err)?
        }

        // Entries
        for e in &st.entries {
            write_entry(&mut wr, e).map_err(map_parse_err)?;
        }

        // </Stmt></BkToCstmrStmt></Document>
        end(&mut wr, "Stmt")?;
        end(&mut wr, "BkToCstmrStmt")?;
        end(&mut wr, "Document")?;
        Ok(())
    }
}

fn write_entry<W: Write>(
    wr: &mut Writer<W>,
    e: &Entry,
) -> std::result::Result<(), quick_xml::Error> {
    wr.write_event(Event::Start(BytesStart::new("Ntry")))?;

    // <NtryRef>REF...</NtryRef>
    if let Some(ref r) = e.reference {
        if !r.is_empty() {
            wr.write_event(Event::Start(BytesStart::new("NtryRef")))?;
            wr.write_event(Event::Text(BytesText::new(r)))?;
            wr.write_event(Event::End(BytesStart::new("NtryRef").to_end()))?;
        }
    }

    // <Amt Ccy="...">...</Amt>
    let amt = e.amount.to_string();
    wr.write_event(Event::Start(
        BytesStart::new("Amt").with_attributes([("Ccy", e.currency.as_str())]),
    ))?;
    wr.write_event(Event::Text(BytesText::new(&amt)))?;
    wr.write_event(Event::End(BytesStart::new("Amt").to_end()))?;

    // <CdtDbtInd>CRDT|DBIT</CdtDbtInd>
    let ind = match e.kind {
        DebitCredit::Credit => "CRDT",
        DebitCredit::Debit => "DBIT",
    };
    wr.write_event(Event::Start(BytesStart::new("CdtDbtInd")))?;
    wr.write_event(Event::Text(BytesText::new(ind)))?;
    wr.write_event(Event::End(BytesStart::new("CdtDbtInd").to_end()))?;

    // <ValDt><Dt>YYYY-MM-DD</Dt></ValDt>
        let vd = e.value_date.clone();
        wr.write_event(Event::Start(BytesStart::new("ValDt")))?;
        wr.write_event(Event::Start(BytesStart::new("Dt")))?;
        wr.write_event(Event::Text(BytesText::new(&vd)))?;
        wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
wr.write_event(Event::End(BytesStart::new("ValDt").to_end()))?;

    // <BookgDt><Dt>YYYY-MM-DD</Dt></BookgDt>
    let bd = e.booking_date.clone();
    wr.write_event(Event::Start(BytesStart::new("BookgDt")))?;
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Text(BytesText::new(&bd)))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("BookgDt").to_end()))?;

    // <AddtlNtryInf>...</AddtlNtryInf>
    if !e.description.is_empty() {
        wr.write_event(Event::Start(BytesStart::new("AddtlNtryInf")))?;
        wr.write_event(Event::Text(BytesText::new(&e.description)))?;
        wr.write_event(Event::End(BytesStart::new("AddtlNtryInf").to_end()))?;
    }

    wr.write_event(Event::End(BytesStart::new("Ntry").to_end()))?;
    Ok(())
}

fn write_balance<W: Write>(
    wr: &mut Writer<W>,
    tp: &str,
    b: &Balance,
) -> std::result::Result<(), quick_xml::Error> {
    wr.write_event(Event::Start(BytesStart::new("Bal")))?;
    wr.write_event(Event::Start(BytesStart::new("Tp")))?;
    wr.write_event(Event::Start(BytesStart::new("CdOrPrtry")))?;
    wr.write_event(Event::Start(BytesStart::new("Cd")))?;
    wr.write_event(Event::Text(BytesText::new(tp)))?;
    wr.write_event(Event::End(BytesStart::new("Cd").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("CdOrPrtry").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("Tp").to_end()))?;

    let amt_str = b.amount.to_string();
    wr.write_event(Event::Start(
        BytesStart::new("Amt").with_attributes([("Ccy", b.currency.as_str())]),
    ))?;
    wr.write_event(Event::Text(BytesText::new(&amt_str)))?;
    wr.write_event(Event::End(BytesStart::new("Amt").to_end()))?;

    let d = b.date_yyymmdd.clone();
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Text(BytesText::new(&d)))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;

    wr.write_event(Event::End(BytesStart::new("Bal").to_end()))?;
    Ok(())
}

/* ====================== Writer helpers ====================== */

type QxRes = Result<(), AdapterError>;

fn start<W: Write>(wr: &mut Writer<W>, name: &str) -> QxRes {
    wr.write_event(Event::Start(BytesStart::new(name))).map_err(map_parse_err)
}

fn end<W: Write>(wr: &mut Writer<W>, name: &str) -> QxRes {
    wr.write_event(Event::End(BytesStart::new(name).to_end())).map_err(map_parse_err)
}

fn text<W: Write>(wr: &mut Writer<W>, s: &str) -> QxRes {
    wr.write_event(Event::Text(BytesText::new(s))).map_err(map_parse_err)
}

/// <name>text</name>
fn elem_text<W: Write>(wr: &mut Writer<W>, name: &str, s: &str) -> QxRes {
    start(wr, name)?;
    text(wr, s)?;
    end(wr, name)
}


/* ====================== Reader helpers ====================== */
fn read_text(e: BytesText<'_>) -> Result<String, AdapterError> {
    let s = std::str::from_utf8(e.as_ref())
        .map_err(|err| AdapterError::ParseError(err.to_string()))?;
    Ok(unescape(s)
        .map_err(|err| AdapterError::ParseError(err.to_string()))?
        .into_owned())
}

fn attr_value(e: &BytesStart<'_>, key: &[u8]) -> Option<String> {
    // безопасно пробуем извлечь значение атрибута
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            if let Ok(v) = String::from_utf8(a.value.into_owned()) {
                return Some(v);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_text_plain() {
        let text = BytesText::from_escaped("Hello World");
        let result = read_text(text).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_read_text_with_escape() {
        let text = BytesText::from_escaped("Tom &amp; Jerry &lt;3");
        let result = read_text(text).unwrap();
        assert_eq!(result, "Tom & Jerry <3");
    }

    #[test]
    fn attr_value_found() {
        let mut el = BytesStart::new("Amt");
        el.push_attribute(("Ccy", "EUR"));
        el.push_attribute(("Scale", "2"));

        let val = attr_value(&el, b"Ccy");
        assert_eq!(val.as_deref(), Some("EUR"));
    }

    #[test]
    fn attr_value_not_found() {
        let mut el = BytesStart::new("Amt");
        el.push_attribute(("Ccy", "EUR"));

        let val = attr_value(&el, b"Missing");
        assert!(val.is_none());
    }

    #[test]
    fn attr_value_multiple_attrs() {
        let mut el = BytesStart::new("Amt");
        el.push_attribute(("Scale", "2"));
        el.push_attribute(("Ccy", "USD"));
        el.push_attribute(("Note", "net"));

        let val = attr_value(&el, b"Ccy");
        assert_eq!(val.as_deref(), Some("USD"));
    }

    #[test]
    fn start_then_end_produces_empty_element_pair() {
        let inner = Vec::<u8>::new();
        let mut writer = Writer::new(inner);

        start(&mut writer, "Tag").unwrap();
        end(&mut writer, "Tag").unwrap();

        let out = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(out, "<Tag></Tag>");
    }

    #[test]
    fn elem_text_writes_wrapped_text() {
        let mut writer = Writer::new(Vec::<u8>::new());
        elem_text(&mut writer, "Id", "123").unwrap();
        let out = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(out, "<Id>123</Id>");
    }

    #[test]
    fn text_is_escaped() {
        let mut writer = Writer::new(Vec::<u8>::new());
        start(&mut writer, "a").unwrap();
        text(&mut writer, "Tom & Jerry <3>").unwrap();
        end(&mut writer, "a").unwrap();

        let out = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(out, "<a>Tom &amp; Jerry &lt;3&gt;</a>");
    }
}