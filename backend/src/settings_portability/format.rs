use crate::{
    domain,
    error::{AppError, Result},
    imports::profiles::Layout,
    rules::Operator,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::{HashMap, HashSet};

pub const MAX_BYTES: usize = 8 * 1024 * 1024;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub format: String,
    pub version: u32,
    pub setup_complete: bool,
    pub classifications: Vec<Classification>,
    pub sources: Vec<Source>,
    pub profiles: Vec<Profile>,
    pub rules: Vec<Rule>,
    pub budgets: Vec<Budget>,
}
#[derive(Deserialize, Serialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub color: String,
    pub archived: bool,
}
#[derive(Deserialize, Serialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub name: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub layout: Layout,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: String,
    pub operator: Operator,
    pub pattern: String,
    pub source_id: Option<String>,
    pub category_id: Option<String>,
    pub label_ids: Vec<String>,
    pub enabled: bool,
    pub pause_reason: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub month: String,
    pub savings_cents: String,
    pub categories: Vec<Target>,
    pub income: Vec<Income>,
}
#[derive(Deserialize, Serialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub category_id: String,
    pub amount_cents: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Income {
    pub name: String,
    pub amount_cents: String,
    pub complete: bool,
    pub entry_refs: Vec<String>,
}
#[derive(Serialize)]
pub struct Summary {
    pub categories: usize,
    pub labels: usize,
    pub sources: usize,
    pub profiles: usize,
    pub rules: usize,
    pub budgets: usize,
    pub receipt_links: usize,
}

fn invalid() -> AppError {
    AppError::invalid(
        "Settings contain invalid or repeated references. Export a fresh settings file.",
    )
}
fn reference(id: &str) -> Result<()> {
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| invalid())?;
    if parsed.to_string() != id {
        return Err(invalid());
    }
    Ok(())
}
pub fn cents(value: &str) -> Result<i64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::invalid(
            "Settings amounts must be nonnegative integer-cent strings.",
        ));
    }
    value
        .parse()
        .map_err(|_| AppError::invalid("A settings amount exceeds the supported range."))
}
impl Settings {
    pub fn validate(&self) -> Result<Summary> {
        if self.format != "coinz-settings" || self.version != 1 {
            return Err(AppError::invalid(
                "Unsupported settings format or version. Use a version 1 coinz settings export.",
            ));
        }
        if self.classifications.len() > 10_000
            || self.sources.len() > 10_000
            || self.profiles.len() > 1_000
            || self.rules.len() > 10_000
            || self.budgets.len() > 1_200
        {
            return Err(AppError::invalid(
                "This settings file exceeds the supported item limits.",
            ));
        }
        let mut items = HashMap::new();
        let mut names = HashSet::new();
        for item in &self.classifications {
            reference(&item.id)?;
            domain::text(&item.name, "Classification name", 60)?;
            if !["category", "label"].contains(&item.kind.as_str())
                || !["sage", "clay", "sand", "lavender", "blue"].contains(&item.color.as_str())
                || items.insert(&item.id, item).is_some()
                || !names.insert((&item.kind, item.name.trim().to_lowercase()))
            {
                return Err(invalid());
            }
        }
        let target = |id: &String, kind: &str, enabled: bool| -> Result<()> {
            let item = items.get(id).ok_or_else(invalid)?;
            if item.kind != kind || (enabled && item.archived) {
                return Err(invalid());
            }
            Ok(())
        };
        let mut sources = HashSet::new();
        for source in &self.sources {
            reference(&source.id)?;
            domain::text(&source.name, "Source name", 80)?;
            if !sources.insert(&source.id) {
                return Err(invalid());
            }
        }
        for profile in &self.profiles {
            domain::text(&profile.name, "Profile name", 80)?;
            profile.layout.validate()?;
        }
        for rule in &self.rules {
            domain::text(&rule.name, "Rule name", 60)?;
            domain::text(&rule.pattern, "Rule pattern", 200)?;
            if matches!(rule.operator, Operator::ContainsWord)
                && !rule.pattern.trim().chars().all(char::is_alphanumeric)
            {
                return Err(invalid());
            }
            if let Some(reason) = &rule.pause_reason {
                domain::text(reason, "Pause reason", 500)?;
            }
            if rule.enabled
                && (rule.pause_reason.is_some()
                    || (rule.category_id.is_none() && rule.label_ids.is_empty()))
            {
                return Err(invalid());
            }
            if let Some(id) = &rule.source_id
                && !sources.contains(id)
            {
                return Err(invalid());
            }
            if let Some(id) = &rule.category_id {
                target(id, "category", rule.enabled)?;
            }
            let mut labels = HashSet::new();
            if rule.label_ids.len() > 100 {
                return Err(invalid());
            }
            for id in &rule.label_ids {
                target(id, "label", rule.enabled)?;
                if !labels.insert(id) {
                    return Err(invalid());
                }
            }
        }
        let mut months = HashSet::new();
        let mut receipts = HashSet::new();
        for budget in &self.budgets {
            domain::month_range(&budget.month)?;
            if !months.insert(&budget.month)
                || budget.categories.len() > 500
                || budget.income.len() > 100
            {
                return Err(invalid());
            }
            let savings = cents(&budget.savings_cents)?;
            let mut categories = HashSet::new();
            let mut amounts = Vec::new();
            for row in &budget.categories {
                target(&row.category_id, "category", false)?;
                if !categories.insert(&row.category_id) {
                    return Err(invalid());
                }
                amounts.push(cents(&row.amount_cents)?);
            }
            let total = domain::checked_total(amounts)?;
            let mut income = Vec::new();
            for row in &budget.income {
                domain::text(&row.name, "Expected income name", 100)?;
                income.push(cents(&row.amount_cents)?);
                if row.entry_refs.len() > 500 {
                    return Err(invalid());
                }
                for id in &row.entry_refs {
                    reference(id)?;
                    if !receipts.insert(id) {
                        return Err(invalid());
                    }
                }
            }
            domain::checked_total([domain::checked_total(income)?, -savings, -total])?;
        }
        Ok(Summary {
            categories: self
                .classifications
                .iter()
                .filter(|c| c.kind == "category")
                .count(),
            labels: self
                .classifications
                .iter()
                .filter(|c| c.kind == "label")
                .count(),
            sources: self.sources.len(),
            profiles: self.profiles.len(),
            rules: self.rules.len(),
            budgets: self.budgets.len(),
            receipt_links: receipts.len(),
        })
    }
}
