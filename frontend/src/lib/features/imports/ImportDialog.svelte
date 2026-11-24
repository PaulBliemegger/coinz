<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    api,
    type Source,
    type ImportRow,
    type ImportOutcome,
    type ImportProfile,
    type CsvSample,
  } from '#lib/api.ts';
  import type { RuleSuggestions } from '#lib/features/rules/types.ts';
  import { money } from '#lib/money.ts';
  import ImportMapping from './ImportMapping.svelte';
  import ImportReviewRow from './ImportReviewRow.svelte';
  import { importSelection } from './selection.ts';
  import type {
    ClassificationCatalog,
    ClassificationChange,
  } from '#lib/features/classification/types.ts';
  import {
    mergeRuleSuggestions,
    updateClassificationDraft,
    selectedClassificationDrafts,
    type ImportClassificationDraft,
  } from './classification.ts';

  let {
    sources,
    catalog,
    oncatalogchanged,
    initialSource,
    onclose,
    onimported,
  }: {
    sources: Source[];
    catalog: ClassificationCatalog;
    oncatalogchanged: () => Promise<void>;
    initialSource: string;
    onclose: () => void;
    onimported: (result: ImportOutcome, source: string) => Promise<void>;
  } = $props();
  const formId = $props.id();
  let nativeFile = $state(false);
  let dialog: HTMLDialogElement;
  let previewHeading = $state<HTMLHeadingElement>();
  let choices = $state<Source[]>([]);
  let sourceId = $state('');
  let sourceName = $state('');
  let newSource = $state(false);
  let file = $state<File | null>(null);
  let csv = $state('');
  let rows = $state<ImportRow[] | null>(null);
  let decisions = $state<Record<number, string>>({});
  let profiles = $state<ImportProfile[]>([]);
  let profileId = $state('');
  let mapping = $state(false);
  let rulesRevision = $state(0);
  let suggestionRecords = $state<number[]>([]);
  let failedSuggestions = $state<number[]>([]);
  let reviewGeneration = 0;
  const suggestionRequests: Record<number, number> = {};
  let classifications = $state<Record<number, ImportClassificationDraft>>({});
  let discardedClassifications = $state<Record<number, boolean>>({});
  let classificationRecords = $state<number[]>([]);
  let bulkAction = $state('');
  let bulkCategory = $state('');
  let bulkLabels = $state<string[]>([]);
  let descriptions = $state<Record<number, string>>({});
  let discardedDescriptions = $state<Record<number, string>>({});
  let discard = $state<'mapping' | 'file' | null>(null);
  let loadingProfiles = $state(true);
  let checkingFile = $state(false);
  let compatible = $state(false);
  let compatibilityError = $state('');
  let compatibilityRequest = 0;
  const profile = $derived(profiles.find((p) => p.id === profileId) ?? null);
  const displayRows = $derived(
    (rows ?? []).map((row) => ({
      ...row,
      description: descriptions[row.record] ?? row.description,
      description_error:
        descriptions[row.record] ||
        row.status === 'known' ||
        decisions[row.record]?.startsWith('match:')
          ? null
          : row.description_error,
    })),
  );
  let busy = $state(false);
  let error = $state('');
  const working = $derived(
    busy || classificationRecords.length > 0 || suggestionRecords.length > 0,
  );
  let selected = $state<number[]>([]);
  let editingRecords = $state<number[]>([]);
  function setClassificationBusy(record: number, value: boolean) {
    classificationRecords = value
      ? [...new Set([...classificationRecords, record])]
      : classificationRecords.filter((id) => id !== record);
  }
  async function editClassification(record: number, change: ClassificationChange) {
    classifications = {
      ...classifications,
      [record]: updateClassificationDraft(classifications[record], change),
    };
  }
  function bulkClassify(event: SubmitEvent) {
    event.preventDefault();
    const change: ClassificationChange =
      bulkAction === 'set_category'
        ? { action: 'set_category', category_id: bulkCategory || null }
        : {
            action: bulkAction === 'add_labels' ? 'add_labels' : 'remove_labels',
            label_ids: bulkLabels,
          };
    const next = { ...classifications };
    for (const row of selection.additions)
      next[row.record] = updateClassificationDraft(next[row.record], change);
    classifications = next;
    bulkAction = '';
    bulkLabels = [];
  }
  function setEditing(record: number, editing: boolean) {
    editingRecords = editing
      ? [...new Set([...editingRecords, record])]
      : editingRecords.filter((id) => id !== record);
  }
  const selection = $derived(importSelection(displayRows, selected, decisions));
  const ruleConflicts = $derived(
    selection.additions.filter(
      (row) => row.rule_suggestions?.conflict && !classifications[row.record]?.category_confirmed,
    ),
  );
  const attention = $derived(
    displayRows.filter(
      (row) =>
        ['invalid', 'conflict'].includes(row.status) ||
        !!row.description_error ||
        (row.status === 'review' && !decisions[row.record]),
    ),
  );
  const readyRows = $derived(
    displayRows.filter(
      (row) =>
        !row.description_error &&
        (row.status === 'new' || (row.status === 'review' && !!decisions[row.record])),
    ),
  );
  const knownRows = $derived(displayRows.filter((row) => row.status === 'known'));
  const eligible = $derived(
    readyRows.filter((row) => row.status === 'new' || decisions[row.record] === 'keep'),
  );
  $effect(() => {
    const chosen = profile;
    const selectedFile = file;
    const request = ++compatibilityRequest;
    compatible = false;
    compatibilityError = '';
    if (!nativeFile && chosen && selectedFile) void checkFile(selectedFile, chosen, request);
    else checkingFile = false;
    return () => {
      compatibilityRequest++;
    };
  });
  async function checkFile(selectedFile: File, chosen: ImportProfile, request: number) {
    checkingFile = true;
    try {
      if (selectedFile.size > 256 * 1024) throw new Error('CSV files must be at most 256 KiB.');
      const text = new TextDecoder('utf-8', { fatal: true }).decode(
        await selectedFile.arrayBuffer(),
      );
      const sample = await api<CsvSample>('/imports/sample', 'POST', {
        csv: text,
        delimiter: chosen.layout.delimiter,
      });
      if (request !== compatibilityRequest) return;
      const headers = [
        chosen.layout.date,
        chosen.layout.amount,
        chosen.layout.description,
        chosen.layout.external_id,
      ].filter(Boolean);
      if (headers.some((header) => sample.headers.filter((value) => value === header).length !== 1))
        throw new Error(
          'A mapped column is missing or ambiguous. Check column mapping before review.',
        );
      compatible = true;
    } catch (cause) {
      if (request === compatibilityRequest)
        compatibilityError =
          cause instanceof Error ? cause.message : 'Could not check file headers.';
    } finally {
      if (request === compatibilityRequest) checkingFile = false;
    }
  }
  function select(record: number, checked: boolean) {
    selected = checked
      ? [...new Set([...selected, record])]
      : selected.filter((id) => id !== record);
  }
  function decide(record: number, choice: string) {
    decisions = { ...decisions, [record]: choice };
    if (choice.startsWith('match:') && descriptions[record] !== undefined) {
      discardedDescriptions = { ...discardedDescriptions, [record]: descriptions[record] };
      const remaining = { ...descriptions };
      delete remaining[record];
      descriptions = remaining;
    }
    if (choice.startsWith('match:') && classifications[record]) {
      discardedClassifications = { ...discardedClassifications, [record]: true };
      const next = { ...classifications };
      delete next[record];
      classifications = next;
    }
    if (choice === 'keep') {
      const plan = rows?.find((row) => row.record === record)?.rule_suggestions;
      if (plan?.matched_rules.length)
        classifications = {
          ...classifications,
          [record]: mergeRuleSuggestions(plan, classifications[record]),
        };
    }
    select(record, choice === 'keep');
  }
  onMount(() => {
    choices = [...sources];
    sourceId = initialSource;
    dialog.showModal();
    void loadProfiles();
  });

  async function loadProfiles() {
    error = '';
    loadingProfiles = true;
    try {
      profiles = await api<ImportProfile[]>('/imports/profiles');
    } catch (cause) {
      fail(cause);
    } finally {
      loadingProfiles = false;
    }
  }
  async function editDescription(record: number, value: string) {
    descriptions = { ...descriptions, [record]: value.trim() };
    const request = (suggestionRequests[record] ?? 0) + 1;
    suggestionRequests[record] = request;
    const generation = reviewGeneration;
    suggestionRecords = [...new Set([...suggestionRecords, record])];
    failedSuggestions = failedSuggestions.filter((id) => id !== record);
    try {
      const plan = await api<RuleSuggestions>('/rules/suggest', 'POST', {
        description: value.trim(),
        source_id: nativeFile ? null : sourceId || null,
      });
      if (generation !== reviewGeneration || suggestionRequests[record] !== request) return;
      if (plan.revision !== rulesRevision)
        throw new Error('Rules changed. Review the file again before importing.');
      rows =
        rows?.map((row) => (row.record === record ? { ...row, rule_suggestions: plan } : row)) ??
        null;
      if (!decisions[record]?.startsWith('match:'))
        classifications = {
          ...classifications,
          [record]: mergeRuleSuggestions(plan, classifications[record]),
        };
    } catch (cause) {
      if (generation === reviewGeneration) {
        failedSuggestions = [...new Set([...failedSuggestions, record])];
        fail(cause);
      }
    } finally {
      if (suggestionRequests[record] === request)
        suggestionRecords = suggestionRecords.filter((id) => id !== record);
    }
    await tick();
    document.getElementById(`import-description-${record}`)?.focus();
  }
  async function review(chosen: ImportProfile | null) {
    const result = await api<{ rows: ImportRow[]; rules_revision: number }>(
      nativeFile ? '/imports/restore/preview' : '/imports/preview',
      'POST',
      {
        source_id: nativeFile ? null : sourceId || null,
        csv,
        profile_id: chosen?.id,
        profile_revision: chosen?.revision,
      },
    );
    ++reviewGeneration;
    suggestionRecords = [];
    failedSuggestions = [];
    rows = result.rows;
    rulesRevision = result.rules_revision;
    descriptions = {};
    classifications = Object.fromEntries(
      result.rows
        .filter((row) => row.status === 'new' && row.rule_suggestions?.matched_rules.length)
        .map((row) => [row.record, mergeRuleSuggestions(row.rule_suggestions!)]),
    );
    discardedClassifications = {};
    bulkAction = '';
    discardedDescriptions = {};
    decisions = {};
    discard = null;
    mapping = false;
    selected = result.rows
      .filter((row) => row.status === 'new' && !row.description_error)
      .map((row) => row.record);
    await tick();
    previewHeading?.focus();
  }
  function leaveReview(target: 'mapping' | 'file') {
    discard = target;
  }
  function resetReview(target: 'mapping' | 'file') {
    ++reviewGeneration;
    suggestionRecords = [];
    failedSuggestions = [];
    rows = null;
    descriptions = {};
    classifications = {};
    discardedClassifications = {};
    bulkAction = '';
    discardedDescriptions = {};
    decisions = {};
    selected = [];
    discard = null;
    error = '';
    mapping = target === 'mapping';
  }
  function fail(cause: unknown) {
    error = cause instanceof Error ? cause.message : 'Could not import this file.';
  }

  async function createSource(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    error = '';
    try {
      const result = await api<{ id: string }>('/sources', 'POST', { name: sourceName });
      choices = [
        ...choices,
        { id: result.id, name: sourceName.trim(), latest_transaction_date: null },
      ];
      sourceId = result.id;
      newSource = false;
      sourceName = '';
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }

  async function chooseFile(chosen: File | null) {
    file = chosen;
    nativeFile = false;
    csv = '';
    error = '';
    if (!chosen) return;
    busy = true;
    try {
      const header = (await chosen.slice(0, 512).text()).split(/\r?\n/, 1)[0];
      if (
        file === chosen &&
        header === 'date,amount,description,category,labels,source,entry_ref,source_ref,origin'
      ) {
        nativeFile = true;
        profileId = '';
        sourceId = '';
        compatibilityError = '';
      }
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }
  async function preview(event: SubmitEvent) {
    event.preventDefault();
    if (!file) return;
    error = '';
    busy = true;
    csv = '';
    try {
      if (file.size > (nativeFile ? 16 * 1024 * 1024 : 256 * 1024))
        throw new Error(
          nativeFile
            ? 'Coinz exports must be at most 16 MiB.'
            : 'CSV files must be at most 256 KiB.',
        );
      csv = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer());
      if (nativeFile || profile) await review(profile);
      else mapping = true;
    } catch (cause) {
      fail(cause);
      if (csv && profile) mapping = true;
    } finally {
      busy = false;
    }
  }

  async function commit() {
    busy = true;
    error = '';
    try {
      const result = await api<ImportOutcome>(
        nativeFile ? '/imports/restore/commit' : '/imports/commit',
        'POST',
        {
          source_id: nativeFile ? null : sourceId || null,
          csv,
          filename: file?.name,
          profile_id: nativeFile ? undefined : profile?.id,
          profile_revision: nativeFile ? undefined : profile?.revision,
          descriptions: Object.fromEntries(
            selection.additions
              .filter((row) => descriptions[row.record] !== undefined)
              .map((row) => [row.record, descriptions[row.record]]),
          ),
          rules_revision: nativeFile ? undefined : rulesRevision,
          classifications: selectedClassificationDrafts(classifications, selection.additions),
          selected_records: selection.records,
          decisions: selection.decisions,
        },
      );
      // Import succeeded. Refresh errors must never invite a duplicate submission.
      dialog.close();
      await onimported(result, sourceId);
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="import-title"
  class="import-dialog"
  class:mapping-workspace={mapping}
  oncancel={(event) => {
    if (working) event.preventDefault();
  }}
  {onclose}
>
  <div class="dialog-heading">
    <div>
      <p class="eyebrow">A LITTLE LESS TYPING</p>
      <h2 id="import-title">Import CSV</h2>
    </div>
    <button
      type="button"
      class="button quiet"
      disabled={working}
      onclick={() => dialog.close()}
      aria-label="Close import"
    >
      ✕
    </button>
  </div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  <ol class="stages" aria-label="Import stages">
    <li aria-current={!mapping && !rows ? 'step' : undefined}>File & profile</li>
    <li aria-current={mapping ? 'step' : undefined}>Map columns</li>
    <li aria-current={rows ? 'step' : undefined}>Review entries</li>
  </ol>
  {#if mapping}
    <div class="review-scroll">
      <p class="muted">
        {file?.name} · {nativeFile
          ? 'Saved per-entry sources'
          : (choices.find((source) => source.id === sourceId)?.name ?? 'No source')}
      </p>
      <ImportMapping
        {csv}
        {sourceId}
        {profile}
        onback={() => (mapping = false)}
        onsaved={async (saved) => {
          profiles = [...profiles.filter((p) => p.id !== saved.id), saved];
          profileId = saved.id;
          await review(saved);
        }}
      />
    </div>
  {:else if !rows}
    <p>Choose a reusable layout, or map your CSV columns to Date, Amount, and Description.</p>
    {#if newSource}
      <form onsubmit={createSource}>
        <label>
          New source name<input
            bind:value={sourceName}
            required
            maxlength="80"
            disabled={working}
          />
        </label>
        <div class="import-actions">
          <button class="button" disabled={working}>Create import source</button><button
            type="button"
            class="button quiet"
            disabled={working}
            onclick={() => (newSource = false)}
          >
            Cancel source
          </button>
        </div>
      </form>
    {:else}
      <form onsubmit={preview}>
        {#if !nativeFile}<details>
            <summary>Optional source</summary>
            <label>
              <span id={`${formId}-source`}>Import source</span><select
                aria-labelledby={`${formId}-source`}
                bind:value={sourceId}
                disabled={working}
              >
                <option value="">No source</option>{#each choices as source (source.id)}<option
                    value={source.id}
                  >
                    {source.name}
                  </option>{/each}
              </select>
            </label>
            <button
              type="button"
              class="button quiet"
              disabled={working}
              onclick={() => (newSource = true)}
            >
              + New source
            </button>
          </details>{/if}
        <label>
          CSV file<input
            type="file"
            accept=".csv,text/csv"
            required={!file}
            disabled={working}
            onchange={(event) => void chooseFile(event.currentTarget.files?.[0] ?? null)}
          />
        </label>
        {#if file}<p class="muted">Selected file: {file.name}</p>{/if}
        {#if nativeFile}<p class="notice" role="status">
            Coinz export detected. Restore saved categories, labels and sources without a profile.
            Up to 16 MiB and 50,000 entries. Missing names are created; existing setup is retained.
            Matching keeps the existing entry unchanged.
          </p>
        {:else}<p class="muted">
            Up to 256 KiB and 1,000 records. Preview first; nothing is added until you confirm.
            Sources are optional metadata and hold no bank balance.
          </p>
          <label>
            <span id={`${formId}-profile`}>Import profile</span><select
              aria-labelledby={`${formId}-profile`}
              bind:value={profileId}
              disabled={working || loadingProfiles}
            >
              <option value="">Create a new profile</option>
              {#each profiles as item (item.id)}<option value={item.id}>{item.name}</option>{/each}
            </select>
          </label>
          {#if loadingProfiles}<p role="status">Loading profiles…</p>{/if}
          <button
            type="button"
            class="button quiet"
            disabled={working || loadingProfiles}
            onclick={loadProfiles}
          >
            Reload profiles
          </button>
          {#if profile}<p class="muted">
              Date: {profile.layout.date} · Amount: {profile.layout.amount} · Description: {profile
                .layout.description}
            </p>
            <button
              type="button"
              class="button quiet"
              disabled={working || !file}
              onclick={async () => {
                busy = true;
                error = '';
                try {
                  if (!file || file.size > 256 * 1024)
                    throw new Error('CSV files must be at most 256 KiB.');
                  csv = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer());
                  mapping = true;
                } catch (cause) {
                  fail(cause);
                } finally {
                  busy = false;
                }
              }}
            >
              Check column mapping
            </button>
            <button
              type="button"
              class="button quiet"
              disabled={working}
              onclick={() => (profileId = '')}
            >
              Create a new profile
            </button>
          {/if}
          {#if compatibilityError}<p class="error" role="alert">{compatibilityError}</p>{/if}
          <p class="muted">
            Saved mappings are checked against this file before review. Saved profiles and sources
            remain if you cancel.
          </p>
        {/if}
        <button
          class="button primary"
          disabled={working ||
            (!nativeFile &&
              (loadingProfiles || checkingFile || (profile !== null && !compatible))) ||
            !file}
        >
          {busy || checkingFile
            ? 'Reading…'
            : nativeFile || profile
              ? 'Review entries'
              : 'Map columns'}
        </button>
      </form>
    {/if}
  {:else}
    <div class="review-scroll">
      <h3 bind:this={previewHeading} tabindex="-1">Review {rows.length} entries</h3>
      <p class="muted">
        {file?.name} · {choices.find((source) => source.id === sourceId)?.name ?? 'No source'}
      </p>
      {#if attention.length}<h3 class="group-heading">Needs attention · {attention.length}</h3>
        {#each attention as row (row.record)}<ImportReviewRow
            {row}
            {catalog}
            {oncatalogchanged}
            classification={classifications[row.record]}
            discardedClassification={discardedClassifications[row.record]}
            onclassification={(change) => editClassification(row.record, change)}
            onclassificationbusy={(value) => setClassificationBusy(row.record, value)}
            draft={descriptions[row.record]}
            discardedDraft={discardedDescriptions[row.record]}
            ondescription={(value) => editDescription(row.record, value)}
            onediting={(editing) => setEditing(row.record, editing)}
            selected={false}
            choice={decisions[row.record] ?? ''}
            {busy}
            onselect={(checked) => select(row.record, checked)}
            onchoice={(choice) => decide(row.record, choice)}
          />{/each}
      {/if}
      <div class="group-heading select-heading">
        <h3>Ready to add</h3>
        <label>
          <input
            type="checkbox"
            disabled={working || !eligible.length}
            checked={eligible.length > 0 && eligible.every((row) => selected.includes(row.record))}
            indeterminate={eligible.some((row) => selected.includes(row.record)) &&
              !eligible.every((row) => selected.includes(row.record))}
            onchange={(event) => {
              selected = event.currentTarget.checked ? eligible.map((row) => row.record) : [];
            }}
          /> Select new entries
        </label>
      </div>
      {#if selection.additions.length && !nativeFile}
        <section
          class="import-classification-bulk"
          aria-label="Classify selected additions"
          data-entry-controls
        >
          <div class="classification-actions">
            <span class="muted">{selection.additions.length} selected additions</span>
            {#each [['set_category', 'Set category'], ['add_labels', 'Add labels'], ['remove_labels', 'Remove labels']] as [action, label] (action)}
              <button
                type="button"
                class="button quiet"
                disabled={working}
                onclick={() => {
                  bulkAction = bulkAction === action ? '' : action;
                  bulkLabels = [];
                }}
              >
                {label}
              </button>
            {/each}
          </div>
          {#if bulkAction}
            <form class="classification-picker" onsubmit={bulkClassify}>
              {#if bulkAction === 'set_category'}
                <label>
                  <span id={`${formId}-bulk-category`}>Category for selected additions</span><select
                    aria-labelledby={`${formId}-bulk-category`}
                    bind:value={bulkCategory}
                    disabled={working}
                  >
                    <option value="">
                      Uncategorized
                    </option>{#each catalog.categories.filter((c) => !c.archived) as item (item.id)}<option
                        value={item.id}
                      >
                        {item.name}
                      </option>{/each}
                  </select>
                </label>
              {:else}
                <fieldset>
                  <legend>
                    {bulkAction === 'add_labels' ? 'Add' : 'Remove'} labels on selected additions
                  </legend>{#each catalog.labels.filter((l) => bulkAction === 'remove_labels' || !l.archived) as item (item.id)}<label
                      class="setup-choice"
                    >
                      <input
                        type="checkbox"
                        value={item.id}
                        bind:group={bulkLabels}
                        disabled={working}
                      />{item.name}
                    </label>{/each}
                </fieldset>
              {/if}
              <div class="classification-actions">
                <button
                  class="button"
                  disabled={working || (bulkAction !== 'set_category' && !bulkLabels.length)}
                >
                  Apply to {selection.additions.length} additions
                </button><button
                  type="button"
                  class="button quiet"
                  disabled={working}
                  onclick={() => (bulkAction = '')}
                >
                  Cancel
                </button>
              </div>
            </form>
          {/if}
        </section>
      {/if}
      {#each readyRows as row (row.record)}<ImportReviewRow
          {row}
          {catalog}
          {oncatalogchanged}
          classification={classifications[row.record]}
          discardedClassification={discardedClassifications[row.record]}
          onclassification={(change) => editClassification(row.record, change)}
          onclassificationbusy={(value) => setClassificationBusy(row.record, value)}
          draft={descriptions[row.record]}
          discardedDraft={discardedDescriptions[row.record]}
          ondescription={(value) => editDescription(row.record, value)}
          onediting={(editing) => setEditing(row.record, editing)}
          selected={selected.includes(row.record)}
          choice={decisions[row.record] ?? ''}
          {busy}
          onselect={(checked) => select(row.record, checked)}
          onchoice={(choice) => decide(row.record, choice)}
        />{/each}
      {#if !readyRows.length}<p class="muted">
          No new entries are ready. You can resolve a duplicate above or choose another file.
        </p>{/if}
      {#if knownRows.length}<details class="known">
          <summary>
            {knownRows.length} already imported · excluded
          </summary>{#each knownRows as row (row.record)}<ImportReviewRow
              {row}
              {catalog}
              {oncatalogchanged}
              classification={classifications[row.record]}
              discardedClassification={discardedClassifications[row.record]}
              onclassification={(change) => editClassification(row.record, change)}
              onclassificationbusy={(value) => setClassificationBusy(row.record, value)}
              draft={descriptions[row.record]}
              discardedDraft={discardedDescriptions[row.record]}
              ondescription={(value) => editDescription(row.record, value)}
              onediting={(editing) => setEditing(row.record, editing)}
              selected={false}
              choice=""
              {busy}
              onselect={() => {}}
              onchoice={() => {}}
            />{/each}
        </details>{/if}
    </div>
    <div class="import-bottom">
      {#if discard}<div role="alert">
          <p>
            Changing {discard === 'mapping' ? 'the mapping' : 'the file'} clears draft descriptions, categories,
            labels, selections, and duplicate decisions.
          </p>
          <div class="import-actions">
            <button
              type="button"
              class="button"
              disabled={working}
              onclick={() => (discard = null)}
            >
              Keep reviewing
            </button><button
              type="button"
              class="button"
              disabled={working}
              onclick={() => resetReview(discard!)}
            >
              Continue changing {discard}
            </button>
          </div>
        </div>{/if}
      {#if !nativeFile}<button
          type="button"
          class="button quiet"
          disabled={working}
          onclick={() => leaveReview('mapping')}
        >
          Check column mapping
        </button>{/if}
      {#if editingRecords.length}<p role="status">
          Finish or cancel pending description changes before importing.
        </p>{/if}
      {#if ruleConflicts.length}<p class="rule-conflict" role="status">
          Choose a category for {ruleConflicts.length} selected entries with conflicting rules.
        </p>{/if}
      <div class="selected-totals">
        <div>
          <span class="muted">Selected income</span><strong>{money(selection.income)}</strong>
        </div>
        <div>
          <span class="muted">Selected spending</span><strong>{money(selection.spending)}</strong>
        </div>
      </div>
      <p class="muted" role="status">
        {selection.additions.length} selected · {selection.matches.length} matches · {selection.excluded}
        left for later · {knownRows.length} already imported
      </p>
      <div class="import-actions">
        <button
          type="button"
          class="button"
          disabled={working}
          onclick={() => {
            leaveReview('file');
          }}
        >
          Choose file again
        </button><button
          type="button"
          class="button primary"
          disabled={working ||
            discard !== null ||
            editingRecords.length > 0 ||
            ruleConflicts.length > 0 ||
            failedSuggestions.some((id) => selection.records.includes(id)) ||
            !selection.records.length}
          onclick={commit}
        >
          {busy
            ? 'Importing…'
            : `Import ${selection.additions.length} entries${selection.matches.length ? ` + ${selection.matches.length} matches` : ''}`}
        </button>
      </div>
    </div>
  {/if}
</dialog>

<style>
  .import-dialog {
    width: min(780px, calc(100% - 2rem));
  }
  .import-dialog.mapping-workspace {
    width: min(1180px, calc(100% - 2rem));
  }
  .import-dialog[open] {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow: hidden;
  }
  .import-dialog > :global(form) {
    overflow-y: auto;
  }
  .stages {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1.5rem;
    padding-left: 1.25rem;
    font-size: 0.875rem;
  }
  .stages [aria-current='step'] {
    font-weight: 700;
    color: var(--accent);
  }
  .review-scroll {
    overflow-y: auto;
    min-height: 0;
    padding-right: 0.3rem;
  }
  .import-bottom {
    flex-shrink: 0;
    border-top: 1px solid var(--line);
    padding-top: 1rem;
    display: grid;
    gap: 0.75rem;
  }
  .selected-totals {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
  }
  .selected-totals strong {
    display: block;
    font-size: 1.3rem;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .group-heading {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }
  .select-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .select-heading label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 44px;
  }
  input[type='checkbox'] {
    width: 19px;
    min-height: 19px;
    accent-color: var(--accent);
  }
  .known {
    margin: 1rem 0;
  }
  summary {
    min-height: 44px;
  }
  .import-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  @media (max-width: 620px) {
    .import-actions > :global(button) {
      flex: 1;
    }
  }
</style>
