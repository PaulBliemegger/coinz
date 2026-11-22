use crate::{
    domain,
    error::{AppError, Result},
};
use serde::Serialize;

pub const MAX_BYTES: usize = 256 * 1024;
pub const MAX_ROWS: usize = 1000;

#[derive(Clone, Serialize)]
pub struct Row {
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

#[cfg(test)]
mod tests {
    use super::*;
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
