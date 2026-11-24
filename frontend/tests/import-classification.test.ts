import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  updateClassificationDraft,
  selectedClassificationDrafts,
  mergeRuleSuggestions,
} from '../src/lib/features/imports/classification.ts';

test('bulk category and label edits preserve unrelated drafts and exclude non-additions from commit', () => {
  const first = updateClassificationDraft(undefined, {
    action: 'set_category',
    category_id: 'food',
  });
  const labeled = updateClassificationDraft(first, {
    action: 'add_labels',
    label_ids: ['trip', 'shared', 'trip'],
  });
  const changed = updateClassificationDraft(labeled, {
    action: 'set_category',
    category_id: 'leisure',
  });
  assert.deepEqual(changed, {
    category_id: 'leisure',
    label_ids: ['trip', 'shared'],
    category_confirmed: true,
    labels_confirmed: true,
  });
  assert.deepEqual(
    updateClassificationDraft(changed, { action: 'remove_labels', label_ids: ['shared'] }),
    {
      category_id: 'leisure',
      label_ids: ['trip'],
      category_confirmed: true,
      labels_confirmed: true,
    },
  );
  assert.deepEqual(first, { category_id: 'food', label_ids: [], category_confirmed: true });
  const drafts = { 2: changed, 3: labeled, 4: labeled };
  assert.deepEqual(selectedClassificationDrafts(drafts, [{ record: 2 }, { record: 5 }]), {
    2: changed,
  });
  assert.deepEqual(selectedClassificationDrafts(drafts, []), {});
});

test('rule refresh preserves explicit category and label choices, including Uncategorized', () => {
  const plan = {
    revision: 1,
    category_id: 'suggested',
    category_candidates: ['suggested'],
    label_ids: ['auto'],
    conflict: false,
    matched_rules: [],
  };
  const suggested = mergeRuleSuggestions(plan);
  assert.equal(suggested.category_id, 'suggested');
  const manualCategory = updateClassificationDraft(suggested, {
    action: 'set_category',
    category_id: null,
  });
  const manualLabels = updateClassificationDraft(manualCategory, {
    action: 'remove_labels',
    label_ids: ['auto'],
  });
  const refreshed = mergeRuleSuggestions(
    { ...plan, category_id: 'other', label_ids: ['auto', 'new'] },
    manualLabels,
  );
  assert.equal(refreshed.category_id, null);
  assert.deepEqual(refreshed.label_ids, []);
  assert.equal(refreshed.category_confirmed, true);
});
