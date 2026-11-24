use super::profiles::Layout;
use crate::{
    domain,
    error::{AppError, Result},
};
use serde::Serialize;

pub const MAX_BYTES: usize = 256 * 1024;
pub const MAX_ROWS: usize = 1000;

#[derive(Clone, Serialize)]
pub struct Row {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portable: Option<super::portable::Metadata>,
    pub record: usize,
    pub date: String,
    pub description: String,
    pub amount_cents: String,
    pub external_id: String,
    pub error: Option<String>,
    #[serde(skip)]
    pub cents: i64,
}

pub fn parse(input: &str) -> Result<Vec<Row>> {
    if input.len() > MAX_BYTES {
        return Err(AppError::invalid("CSV files must be at most 256 KiB."));
    }
    let mut reader = csv::ReaderBuilder::new().from_reader(input.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| AppError::invalid("Could not read CSV headers."))?;
    if headers.iter().collect::<Vec<_>>() != ["date", "description", "amount", "external_id"] {
        return Err(AppError::invalid(
            "Use these CSV headers in order: date,description,amount,external_id.",
        ));
    }
    let mut rows = Vec::new();
    for (i, record) in reader.records().enumerate() {
        if i >= MAX_ROWS {
            return Err(AppError::invalid("Import at most 1,000 records at once."));
        }
        let record = record.map_err(|_| {
            AppError::invalid(format!(
                "CSV record {} must have four correctly quoted fields.",
                i + 2
            ))
        })?;
        let mut row = Row {
            portable: None,
            record: i + 2,
            date: record[0].trim().into(),
            description: record[1].trim().into(),
            amount_cents: String::new(),
            external_id: record[3].trim().into(),
            error: None,
            cents: 0,
        };
        let validate = || -> Result<i64> {
            domain::date(&row.date)?;
            domain::text(&row.description, "Description", 200)?;
            if row.external_id.chars().count() > 200 {
                return Err(AppError::invalid(
                    "External IDs must be at most 200 characters.",
                ));
            }
            if record[2].contains(',') {
                return Err(AppError::invalid(
                    "Use a dot decimal separator without grouping.",
                ));
            }
            let cents = domain::parse_money(&record[2])?;
            if cents == 0 {
                return Err(AppError::invalid("Amount must not be zero."));
            }
            Ok(cents)
        };
        match validate() {
            Ok(cents) => {
                row.cents = cents;
                row.amount_cents = cents.to_string();
            }
            Err(error) => row.error = Some(error.1),
        }
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(AppError::invalid("The CSV has no transaction records."));
    }
    Ok(rows)
}

// Sample and mapped parsing use the same bounded CSV reader as final interpretation.
#[derive(Serialize)]
pub struct Sample {
    pub headers: Vec<String>,
    pub sample: Vec<Vec<String>>,
    pub total: usize,
}
fn records(input: &str, delimiter: &str) -> Result<(Vec<String>, Vec<csv::StringRecord>)> {
    if input.len() > MAX_BYTES {
        return Err(AppError::invalid("CSV files must be at most 256 KiB."));
    }
    let delimiter = match delimiter {
        "," => b',',
        ";" => b';',
        _ => return Err(AppError::invalid("Choose comma or semicolon delimiter.")),
    };
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(input.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| AppError::invalid("Could not read CSV headers."))?
        .iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if headers.is_empty()
        || headers.len() > 100
        || headers
            .iter()
            .any(|h| h.trim().is_empty() || h.chars().count() > 200)
    {
        return Err(AppError::invalid(
            "Use 1–100 named CSV columns, with headers up to 200 characters.",
        ));
    }
    let mut records = Vec::new();
    for item in reader.records() {
        if records.len() >= MAX_ROWS {
            return Err(AppError::invalid("Import at most 1,000 records at once."));
        }
        records.push(item.map_err(|_| {
            AppError::invalid(
                "CSV records must have the same number of correctly quoted fields as the header.",
            )
        })?);
    }
    if records.is_empty() {
        return Err(AppError::invalid("The CSV has no transaction records."));
    }
    Ok((headers, records))
}
pub fn sample(input: &str, delimiter: &str) -> Result<Sample> {
    let (headers, rows) = records(input, delimiter)?;
    Ok(Sample {
        headers,
        sample: rows
            .iter()
            .take(5)
            .map(|r| r.iter().map(str::to_owned).collect())
            .collect(),
        total: rows.len(),
    })
}
fn column(headers: &[String], name: &str) -> Result<usize> {
    let found = headers
        .iter()
        .enumerate()
        .filter(|(_, h)| h.as_str() == name)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    match found.as_slice() {
        [i] => Ok(*i),
        _ => Err(AppError::invalid(
            "A mapped column is missing or has duplicate headers. Check column mapping.",
        )),
    }
}
fn amount(input: &str, layout: &Layout) -> Result<i64> {
    let input = input.trim();
    let (sign, digits) = input.strip_prefix('-').map_or(("", input), |s| ("-", s));
    let parts: Vec<_> = digits.split(&layout.decimal).collect();
    if parts.len() > 2 {
        return Err(AppError::invalid("Use the confirmed decimal separator."));
    }
    let whole = parts[0];
    if !layout.grouping.is_empty() && whole.contains(&layout.grouping) {
        let groups: Vec<_> = whole.split(&layout.grouping).collect();
        if groups[0].is_empty()
            || groups[0].len() > 3
            || groups.iter().skip(1).any(|s| s.len() != 3)
        {
            return Err(AppError::invalid(
                "Use groups of three digits before the decimal separator.",
            ));
        }
    }
    let whole = if layout.grouping.is_empty() {
        whole.to_owned()
    } else {
        whole.replace(&layout.grouping, "")
    };
    if !whole.bytes().all(|b| b.is_ascii_digit())
        || whole.is_empty()
        || parts
            .get(1)
            .is_some_and(|s| !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(AppError::invalid(
            "Use digits with the confirmed decimal and grouping separators.",
        ));
    }
    let normalized = format!(
        "{sign}{whole}{}",
        parts.get(1).map_or(String::new(), |s| format!(".{s}"))
    );
    let cents = domain::parse_money(&normalized)?;
    if cents == 0 {
        return Err(AppError::invalid("Amount must not be zero."));
    }
    Ok(cents)
}
pub fn mapped(input: &str, layout: &Layout) -> Result<Vec<Row>> {
    layout.validate()?;
    let (headers, records) = records(input, &layout.delimiter)?;
    let date = column(&headers, &layout.date)?;
    let description = column(&headers, &layout.description)?;
    let value = column(&headers, &layout.amount)?;
    let external = layout
        .external_id
        .as_ref()
        .map(|h| column(&headers, h))
        .transpose()?;
    let format = match layout.date_format.as_str() {
        "DD.MM.YYYY" => "%d.%m.%Y",
        "DD/MM/YYYY" => "%d/%m/%Y",
        _ => "%Y-%m-%d",
    };
    Ok(records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let mut row = Row {
                portable: None,
                record: i + 2,
                date: record[date].trim().into(),
                description: record[description].trim().into(),
                external_id: external.map_or("", |i| record[i].trim()).into(),
                amount_cents: String::new(),
                cents: 0,
                error: None,
            };
            let validate = |row: &mut Row| -> Result<()> {
                let parsed = chrono::NaiveDate::parse_from_str(&row.date, format)
                    .map_err(|_| AppError::invalid("Use a valid date in the confirmed format."))?;
                if parsed.format(format).to_string() != row.date {
                    return Err(AppError::invalid("Use the exact confirmed date format."));
                }
                row.date = parsed.format("%Y-%m-%d").to_string();
                domain::date(&row.date)?;
                if row.external_id.chars().count() > 200 {
                    return Err(AppError::invalid(
                        "External IDs must be at most 200 characters.",
                    ));
                }
                row.cents = amount(&record[value], layout)?;
                row.amount_cents = row.cents.to_string();
                Ok(())
            };
            if let Err(e) = validate(&mut row) {
                row.error = Some(e.1);
            }
            // Description validation is separate: review may supply missing text without altering evidence.
            row
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn layout() -> Layout {
        Layout {
            date: "Booking date".into(),
            amount: "Value".into(),
            description: "Name".into(),
            external_id: Some("ID".into()),
            delimiter: ";".into(),
            date_format: "DD.MM.YYYY".into(),
            decimal: ",".into(),
            grouping: ".".into(),
        }
    }
    #[test]
    fn mapped_headers_formats_and_exact_cents() {
        let layout = layout();
        let rows = mapped("ID;Name;Value;Booking date;Unused\r\nx;Example Cafe;-1.234,56;03.11.2026;ignored\r\ny;;2,01;04.11.2026;ignored\r\n", &layout).unwrap();
        assert_eq!(rows[0].date, "2026-11-03");
        assert_eq!(rows[0].cents, -123456);
        assert_eq!(rows[0].external_id, "x");
        assert_eq!(rows[1].description, "");
        assert_eq!(rows[1].cents, 201);
        assert!(
            rows[1].error.is_none(),
            "Description can be supplied in review"
        );
        for value in [
            "1.23,45",
            "1.234,567",
            "1,",
            "1,2,3",
            "1e2",
            "0",
            "+1",
            "1 234,56",
            "92233720368547758,08",
            "-92233720368547758,08",
        ] {
            let csv = format!("ID;Name;Value;Booking date\nx;Cafe;{value};03.11.2026\n");
            assert!(
                mapped(&csv, &layout).unwrap()[0].error.is_some(),
                "accepted {value}"
            );
        }
        for date in ["3.11.2026", "29.02.2026", "0000-01-01"] {
            assert!(
                mapped(
                    &format!("ID;Name;Value;Booking date\nx;Cafe;1;{date}\n"),
                    &layout
                )
                .unwrap()[0]
                    .error
                    .is_some()
            );
        }
        let mut slash = layout.clone();
        slash.date_format = "DD/MM/YYYY".into();
        assert_eq!(
            mapped("ID;Name;Value;Booking date\nx;Cafe;1;03/11/2026\n", &slash).unwrap()[0].date,
            "2026-11-03"
        );
        assert!(
            mapped(
                "ID;Name;Value;Booking date;Name\nx;Cafe;1;03.11.2026;Other\n",
                &layout
            )
            .is_err()
        );
        assert!(
            mapped(
                "ID;Other;Value;Booking date\nx;Cafe;1;03.11.2026\n",
                &layout
            )
            .is_err()
        );
        let mut invalid = layout.clone();
        invalid.grouping = ",".into();
        assert!(invalid.validate().is_err());
        invalid = layout.clone();
        invalid.amount = "Name".into();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn samples_keep_original_values_and_optional_ids_can_be_unmapped() {
        let mut layout = layout();
        layout.delimiter = ",".into();
        layout.date_format = "YYYY-MM-DD".into();
        layout.decimal = ".".into();
        layout.grouping = " ".into();
        layout.external_id = None;
        let csv = "Name,Booking date,Value,Unused,Unused\r\n\"Example, Cafe\",2026-11-03,-1 234.56,x,y\r\n";
        let original = sample(csv, ",").unwrap();
        assert_eq!(original.sample[0][2], "-1 234.56");
        assert_eq!(original.total, 1);
        let rows = mapped(csv, &layout).unwrap();
        assert_eq!(rows[0].description, "Example, Cafe");
        assert_eq!(rows[0].amount_cents, "-123456");
        assert_eq!(rows[0].external_id, "");
        assert!(
            mapped(
                &format!(
                    "Name,Booking date,Value\n{}",
                    "Cafe,2026-11-03,1\n".repeat(MAX_ROWS + 1)
                ),
                &layout
            )
            .is_err()
        );
        let sample = sample(
            &format!(
                "Name,Booking date,Value\n{}",
                "Cafe,2026-11-03,1\n".repeat(7)
            ),
            ",",
        )
        .unwrap();
        assert_eq!(sample.sample.len(), 5);
        assert_eq!(sample.total, 7);
    }

    #[test]
    fn rejects_record_limits_and_invalid_amounts_without_rounding() {
        let header = "date,description,amount,external_id\n";
        assert!(
            parse(&format!(
                "{header}{}",
                "2026-11-01,Example,-1,\n".repeat(MAX_ROWS + 1)
            ))
            .is_err()
        );
        assert!(parse(header).is_err());
        for amount in [
            "0",
            "-0.00",
            "1.001",
            "NaN",
            "1e2",
            "+1",
            "92233720368547758.08",
        ] {
            let rows = parse(&format!("{header}2026-11-01,Example,{amount},\n")).unwrap();
            assert!(rows[0].error.is_some(), "accepted {amount}");
        }
        let rows = parse(&format!("{header}2026-11-01,Example,\"12,34\",\n")).unwrap();
        assert!(rows[0].error.is_some());
    }

    #[test]
    fn parses_quoted_fields_crlf_and_reports_invalid_rows() {
        let rows = parse("date,description,amount,external_id\r\n2026-11-01,\"Books, paper\",-12.34,a\r\n2026-02-30,Bad,1,b\r\n2026-11-01,Bad,1.234,c\r\n").unwrap();
        assert_eq!(rows[0].description, "Books, paper");
        assert_eq!(rows[0].cents, -1234);
        assert!(rows[1].error.is_some());
        assert!(rows[2].error.is_some());
        assert!(parse("date,description,amount,external_id\n2026-11-01,Missing,1\n").is_err());
        assert!(parse("wrong,headers\n").is_err());
        assert!(parse(&"x".repeat(MAX_BYTES + 1)).is_err());
    }
}
