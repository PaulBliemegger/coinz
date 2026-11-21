use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

use crate::error::{AppError, Result};

/// Accept plain euros with one optional decimal separator. Never round input.
pub fn parse_money(input: &str) -> Result<i64> {
    let input = input.trim();
    let (negative, digits) = match input.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, input),
    };
    let mut parts = digits.split(['.', ',']);
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if whole.is_empty() || !whole.bytes().all(|c| c.is_ascii_digit()) || parts.next().is_some() {
        return Err(AppError::invalid(
            "Use an amount such as 12.34 or 12,34, without thousands separators.",
        ));
    }
    let decimal = match fraction {
        None => 0,
        Some(s) if !s.is_empty() && s.len() <= 2 && s.bytes().all(|c| c.is_ascii_digit()) => {
            s.parse::<i64>().map_err(AppError::internal)? * if s.len() == 1 { 10 } else { 1 }
        }
        _ => {
            return Err(AppError::invalid(
                "Amounts can have at most two decimal places.",
            ));
        }
    };
    let cents = whole
        .parse::<i64>()
        .ok()
        .and_then(|n| n.checked_mul(100))
        .and_then(|n| n.checked_add(decimal))
        .ok_or_else(|| AppError::invalid("That amount is too large."))?;
    Ok(if negative { -cents } else { cents })
}

pub fn date(input: &str) -> Result<NaiveDate> {
    let parsed = NaiveDate::parse_from_str(input, "%Y-%m-%d")
        .map_err(|_| AppError::invalid("Use a valid date in YYYY-MM-DD format."))?;
    if parsed.format("%Y-%m-%d").to_string() != input || !(1..=9999).contains(&parsed.year()) {
        return Err(AppError::invalid("Use a valid date in YYYY-MM-DD format."));
    }
    Ok(parsed)
}

pub fn month_range(month: &str) -> Result<(String, String)> {
    let start = date(&format!("{month}-01"))?;
    // Exclusive lexical sentinel above every date in our supported calendar range.
    if month == "9999-12" {
        return Ok((start.to_string(), "9999-13-01".into()));
    }
    let next = if start.month() == 12 {
        NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1)
    }
    .filter(|d| d.year() <= 9999)
    .ok_or_else(|| AppError::invalid("That month is outside the supported range."))?;
    Ok((start.to_string(), next.to_string()))
}

pub fn text(input: &str, label: &str, max: usize) -> Result<String> {
    let input = input.trim();
    if input.is_empty() || input.chars().count() > max {
        return Err(AppError::invalid(format!(
            "{label} must contain between 1 and {max} characters."
        )));
    }
    Ok(input.into())
}

pub fn checked_total(values: impl IntoIterator<Item = i64>) -> Result<i64> {
    let total: i128 = values.into_iter().map(i128::from).sum();
    i64::try_from(total)
        .map_err(|_| AppError::invalid("These entries would exceed the supported balance range."))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Income,
    Expense,
}

impl Kind {
    pub fn amount(&self, value: &str) -> Result<i64> {
        let cents = parse_money(value)?;
        if cents <= 0 {
            return Err(AppError::invalid(
                "Enter an amount greater than zero; choose income or expense separately.",
            ));
        }
        Ok(match self {
            Self::Income => cents,
            Self::Expense => -cents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_euros_exactly_without_rounding() {
        for (input, cents) in [
            ("0.01", 1),
            ("12,34", 1234),
            (" 12.3 ", 1230),
            ("-1000", -100000),
            ("92233720368547758.07", i64::MAX),
        ] {
            assert_eq!(parse_money(input).unwrap(), cents);
        }
        for input in [
            "",
            ".12",
            "12.",
            "12.345",
            "1,000.00",
            "1e2",
            "NaN",
            "+1",
            "€12",
            "92233720368547758.08",
            "-92233720368547758.08",
        ] {
            assert!(parse_money(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn handles_leap_dates_and_exclusive_month_end() {
        assert!(date("2024-02-29").is_ok());
        for invalid in ["2026-02-29", "2026-2-01", "2026-11-31", "0000-01-01"] {
            assert!(date(invalid).is_err());
        }
        assert_eq!(
            month_range("2026-12").unwrap(),
            ("2026-12-01".into(), "2027-01-01".into())
        );
        assert!(month_range("2026-1").is_err());
    }

    #[test]
    fn rejects_total_overflow_but_allows_cancellation() {
        assert!(checked_total([i64::MAX, 1]).is_err());
        assert_eq!(checked_total([i64::MAX, 1, -1]).unwrap(), i64::MAX);
    }
}
