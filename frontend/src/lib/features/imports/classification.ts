import type { RuleSuggestions } from '#lib/features/rules/types.ts';
import type { ClassificationChange } from '#lib/features/classification/types.ts';

export interface ImportClassificationDraft {
  category_id: string | null;
  label_ids: string[];
  category_confirmed?: boolean;
  labels_confirmed?: boolean;
}

export function updateClassificationDraft(
  current: ImportClassificationDraft | undefined,
  change: ClassificationChange,
): ImportClassificationDraft {
  const draft = current ?? { category_id: null, label_ids: [] };
  if (change.action === 'set_category')
    return { ...draft, category_id: change.category_id, category_confirmed: true };
  return {
    ...draft,
    labels_confirmed: true,
    label_ids:
      change.action === 'add_labels'
        ? [...new Set([...draft.label_ids, ...change.label_ids])]
        : draft.label_ids.filter((id) => !change.label_ids.includes(id)),
  };
}

export function selectedClassificationDrafts(
  drafts: Record<number, ImportClassificationDraft>,
  additions: { record: number }[],
) {
  return Object.fromEntries(
    additions
      .filter((row) => drafts[row.record] !== undefined)
      .map((row) => [row.record, drafts[row.record]]),
  );
}

export function mergeRuleSuggestions(
  plan: RuleSuggestions,
  current?: ImportClassificationDraft,
): ImportClassificationDraft {
  return {
    category_id: current?.category_confirmed ? current.category_id : plan.category_id,
    label_ids: current?.labels_confirmed ? current.label_ids : [...plan.label_ids],
    category_confirmed: current?.category_confirmed ?? false,
    labels_confirmed: current?.labels_confirmed ?? false,
  };
}
