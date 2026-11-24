use super::{Decision, Input, Preview};
use crate::{
    classification::{self, Kind},
    error::{AppError, Result},
};
use serde::Deserialize;
use sqlx::SqliteConnection;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Draft {
    category_id: Option<String>,
    #[serde(default)]
    label_ids: Vec<String>,
    #[serde(default)]
    category_confirmed: bool,
    #[serde(default, rename = "labels_confirmed")]
    _labels_confirmed: bool,
}

pub(super) async fn validate(
    conn: &mut SqliteConnection,
    owner: &str,
    input: &Input,
    preview: &Preview,
) -> Result<()> {
    for (record, draft) in &input.classifications {
        let row = preview
            .rows
            .iter()
            .find(|row| row.row.record == *record)
            .ok_or_else(|| {
                AppError::invalid(
                    "Provide classification drafts only for valid CSV record numbers.",
                )
            })?;
        if row.row.portable.is_some() {
            return Err(AppError::invalid(
                "Coinz exports restore their saved classification. Edit it after importing.",
            ));
        }
        if input
            .selected_records
            .as_ref()
            .is_some_and(|selected| !selected.contains(record))
        {
            return Err(AppError::invalid(
                "Provide classification drafts only for selected additions.",
            ));
        }
        let decision = input
            .decisions
            .iter()
            .find(|decision| decision.record() == *record);
        // A retried commit may now be known. Its draft must never overwrite the stored entry.
        if row.status != "known"
            && !matches!(
                (row.status, decision),
                ("new", None | Some(Decision::Keep { .. }))
                    | ("review", Some(Decision::Keep { .. }))
            )
        {
            return Err(AppError::invalid(
                "Classification drafts apply only to additions. Match existing entries without changing their classification.",
            ));
        }
        if input.rules_revision.is_some()
            && row.rule_suggestions.conflict
            && !draft.category_confirmed
        {
            return Err(AppError::invalid(
                "Rules suggest different categories. Choose one, including Uncategorized if appropriate.",
            ));
        }
        classification::validate_ids(&draft.label_ids, true)?;
        if let Some(id) = &draft.category_id {
            classification::owned_item(conn, owner, id, Kind::Category).await?;
        }
        for id in &draft.label_ids {
            classification::owned_item(conn, owner, id, Kind::Label).await?;
        }
    }
    Ok(())
}

pub(super) async fn apply(
    conn: &mut SqliteConnection,
    owner: &str,
    entry: &str,
    draft: &Draft,
) -> Result<()> {
    classification::set_category(conn, owner, entry, draft.category_id.as_deref()).await?;
    for id in &draft.label_ids {
        classification::change_label(conn, owner, entry, id, true).await?;
    }
    Ok(())
}
