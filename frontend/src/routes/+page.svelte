<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { resolve } from '$app/paths';
  import {
    api,
    ApiError,
    setCsrf,
    ledgerQuery,
    type Account,
    type Session,
    type Summary,
    type Transaction,
    type User,
  } from '#lib/api.ts';
  import { amountInput, dateLabel, localDate, money, monthLabel } from '#lib/money.ts';
  import './app.css';

  let user = $state<User | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  let loading = $state(false);
  let error = $state('');
  let formError = $state('');
  let notice = $state('');
  let accounts = $state<Account[]>([]);
  let transactions = $state<Transaction[]>([]);
  let summary = $state<Summary | null>(null);
  let month = $state(localDate().slice(0, 7));
  let accountFilter = $state('');
  let accountDialog: HTMLDialogElement;
  let transactionDialog: HTMLDialogElement;
  let editing = $state<Transaction | null>(null);
  let deleting = $state(false);
  let entryAccount = $state('');
  let entryDate = $state(localDate());
  let entryKind = $state('expense');
  let entryAmount = $state('');
  let entryDescription = $state('');
  let loadVersion = 0;

  const message = (value: unknown) =>
    value instanceof Error ? value.message : 'Something went wrong. Please try again.';

  async function bootstrap() {
    error = '';
    ready = false;
    try {
      const session = await api<Session>('/session');
      setCsrf(session.csrf_token);
      user = session.user;
      if (user) await loadData();
    } catch (cause) {
      error = message(cause);
    } finally {
      ready = true;
    }
  }
  onMount(() => {
    void bootstrap();
  });

  async function loadData() {
    const version = ++loadVersion;
    loading = true;
    error = '';
    const query = ledgerQuery(month, accountFilter);
    try {
      const [nextAccounts, nextTransactions, nextSummary] = await Promise.all([
        api<Account[]>('/accounts'),
        api<Transaction[]>(`/transactions?${query}`),
        api<Summary>(`/summary?${query}`),
      ]);
      if (version !== loadVersion) return;
      accounts = nextAccounts;
      transactions = nextTransactions;
      summary = nextSummary;
    } catch (cause) {
      if (version !== loadVersion) return;
      if (cause instanceof ApiError && cause.status === 401) {
        user = null;
        accounts = [];
        transactions = [];
        summary = null;
      }
      error = message(cause);
    } finally {
      if (version === loadVersion) loading = false;
    }
  }

  async function login(event: SubmitEvent) {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    busy = true;
    error = '';
    try {
      // Refresh anonymous state as well, so expired or logged-out tabs can log in again.
      const anonymous = await api<Session>('/session');
      setCsrf(anonymous.csrf_token);
      const session = await api<Session>('/login', 'POST', {
        username: form.get('username'),
        password: form.get('password'),
      });
      setCsrf(session.csrf_token);
      user = session.user;
      accountFilter = '';
      await loadData();
    } catch (cause) {
      error = message(cause);
    } finally {
      busy = false;
    }
  }

  async function logout() {
    busy = true;
    try {
      await api('/logout', 'POST');
      ++loadVersion;
      user = null;
      accounts = [];
      transactions = [];
      summary = null;
      notice = '';
      await bootstrap();
    } catch (cause) {
      error = message(cause);
    } finally {
      busy = false;
    }
  }

  function openAccount() {
    formError = '';
    accountDialog.showModal();
  }

  async function saveAccount(event: SubmitEvent) {
    event.preventDefault();
    const form = event.currentTarget as HTMLFormElement;
    const values = new FormData(form);
    busy = true;
    formError = '';
    try {
      await api('/accounts', 'POST', Object.fromEntries(values));
      accountDialog.close();
      form.reset();
      notice = 'Account added. A place for your coinz to call home.';
      await loadData();
    } catch (cause) {
      formError = message(cause);
    } finally {
      busy = false;
    }
  }

  async function openEntry(transaction: Transaction | null = null) {
    editing = transaction;
    entryAccount = transaction?.account_id ?? (accountFilter || accounts[0]?.id || '');
    entryDate = transaction?.date ?? localDate();
    entryKind = transaction && BigInt(transaction.amount_cents) > 0n ? 'income' : 'expense';
    entryAmount = transaction ? amountInput(transaction.amount_cents) : '';
    entryDescription = transaction?.description ?? '';
    formError = '';
    deleting = false;
    await tick();
    transactionDialog.showModal();
  }

  async function saveEntry(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    formError = '';
    try {
      await api(
        editing ? `/transactions/${editing.id}` : '/transactions',
        editing ? 'PUT' : 'POST',
        {
          account_id: entryAccount,
          date: entryDate,
          kind: entryKind,
          amount: entryAmount,
          description: entryDescription,
        },
      );
      transactionDialog.close();
      notice = editing ? 'Transaction updated.' : 'Transaction recorded.';
      const entryMonth = entryDate.slice(0, 7);
      if (entryMonth !== month) {
        month = entryMonth;
        notice += ` Showing ${monthLabel(month)}.`;
      }
      if (accountFilter && accountFilter !== entryAccount) accountFilter = entryAccount;
      await loadData();
    } catch (cause) {
      formError = message(cause);
    } finally {
      busy = false;
    }
  }

  async function deleteEntry() {
    if (!editing) return;
    busy = true;
    formError = '';
    try {
      await api(`/transactions/${editing.id}`, 'DELETE');
      transactionDialog.close();
      notice = 'Transaction deleted.';
      await loadData();
    } catch (cause) {
      formError = message(cause);
    } finally {
      busy = false;
    }
  }

  function changeFilter() {
    notice = '';
    void loadData();
  }
</script>

<svelte:head>
  <title>coinz · A little clarity</title>
  <meta
    name="description"
    content="Your personal notebook for income, spending, and a little clarity."
  />
</svelte:head>

{#if !ready}
  <main class="loading-page">
    <span class="wordmark">coinz<span>.</span></span>
    <p>Counting our coinz…</p>
  </main>
{:else if !user}
  <main class="login-page">
    <section class="welcome" aria-labelledby="welcome-heading">
      <span class="wordmark">coinz<span>.</span></span>
      <p class="eyebrow">A PERSONAL FINANCE NOTEBOOK</p>
      <h1 id="welcome-heading">A little clarity.<br />A little less guessing.</h1>
      <p>Make sense of what comes in, what goes out, and where it all wanders off to.</p>
      <span class="welcome-note">Your numbers. At your pace.</span>
    </section>
    <section class="login-panel" aria-labelledby="login-heading">
      <p class="eyebrow">WELCOME BACK</p>
      <h2 id="login-heading">Settle in.</h2>
      <p class="muted">Your notebook is right where you left it.</p>
      <form onsubmit={login}>
        <label>
          Username<input name="username" autocomplete="username" required maxlength="80" />
        </label>
        <label>
          Password<input
            name="password"
            type="password"
            autocomplete="current-password"
            required
            maxlength="128"
          />
        </label>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        <button class="button primary full" disabled={busy}>
          {busy ? 'Signing in…' : 'Log in'}
        </button>
      </form>
      <p class="setup-note">
        First visit? Create your local user with the command in the project README.
      </p>
    </section>
  </main>
{:else}
  <div class="app-shell">
    <header class="topbar">
      <a href={resolve('/')} class="wordmark" aria-label="coinz home">coinz<span>.</span></a>
      <span class="tagline">A little clarity for your everyday.</span>
      <div class="user-menu">
        <span>{user.username}</span><button class="button quiet" onclick={logout} disabled={busy}>
          Log out
        </button>
      </div>
    </header>
    <main class="notebook" aria-busy={loading}>
      <div class="page-heading">
        <div>
          <p class="eyebrow">YOUR MONEY NOTEBOOK</p>
          <h1>Where did it go?</h1>
          <p class="muted">A closer look at the entries you've recorded.</p>
        </div>
        <button
          class="button primary"
          onclick={() => openEntry()}
          disabled={accounts.length === 0 || busy}
        >
          + Add transaction
        </button>
      </div>
      {#if error}<div class="error banner" role="alert">
          {error}<button class="button quiet" onclick={loadData} disabled={loading}>Retry</button>
        </div>{/if}
      {#if notice}<p class="notice" role="status">{notice}</p>{/if}
      <section class="period-controls" aria-label="Transaction filters">
        <label>
          Month<input
            aria-label="Month"
            type="month"
            min="0001-01"
            max="9999-12"
            bind:value={month}
            onchange={changeFilter}
            required
          />
        </label>
        <label>
          Account<select bind:value={accountFilter} onchange={changeFilter}>
            <option value="">All accounts</option>{#each accounts as account (account.id)}<option
                value={account.id}
              >
                {account.name}
              </option>{/each}
          </select>
        </label>
        <span class="scope-note">{loading ? 'Updating…' : 'Recorded transactions · EUR'}</span>
      </section>
      {#if summary && !loading && !error}
        <section class="month-summary" aria-label="Monthly summary">
          <div class="spending-total">
            <p class="eyebrow">RECORDED SPENDING</p>
            <p class="hero-amount" data-testid="spending">{money(summary.spending_cents)}</p>
            <p class="muted">
              {monthLabel(summary.month)} · {summary.transaction_count}
              {summary.transaction_count === 1 ? 'entry' : 'entries'}
            </p>
          </div>
          <div class="secondary-total">
            <p>Recorded income</p>
            <strong data-testid="income">{money(summary.income_cents)}</strong><span class="muted">
              Same period and account selection
            </span>
          </div>
          <div class="secondary-total">
            <p>Income minus spending</p>
            <strong data-testid="net">{money(summary.net_cents, true)}</strong><span class="muted">
              From recorded entries
            </span>
          </div>
        </section>
      {/if}
      <p class="data-note">
        These figures reflect your records, not a live bank connection. Missing entries can change
        the picture.
      </p>
      <div class="workspace">
        <section class="ledger" aria-labelledby="ledger-heading">
          <div class="section-heading">
            <h2 id="ledger-heading">The entries</h2>
            <span class="muted">{month ? monthLabel(month) : 'Choose a month'}</span>
          </div>
          {#if loading}<div class="empty-state"><p>Updating your notebook…</p></div>
          {:else if error}<div class="empty-state">
              <p>The entries couldn't be loaded.</p>
              <p class="muted">Use Retry above to get the latest recorded figures.</p>
            </div>
          {:else if accounts.length === 0}<div class="empty-state">
              <span class="empty-mark" aria-hidden="true">c.</span>
              <h3>Let's give your coinz a home.</h3>
              <p>
                Add an account and its opening balance.<br />Then you can start recording
                transactions.
              </p>
              <button class="button" onclick={openAccount}>Add your first account</button>
            </div>
          {:else if transactions.length === 0}<div class="empty-state">
              <h3>Nothing recorded for this period.</h3>
              <p>Your money is maintaining a low profile.</p>
              <button class="button" onclick={() => openEntry()}>Add a transaction</button>
            </div>
          {:else}
            <table>
              <caption class="sr-only">
                Recorded transactions for {monthLabel(month)}{accountFilter
                  ? ' in the selected account'
                  : ' across all accounts'}
              </caption><thead>
                <tr>
                  <th>Date</th><th>Description</th><th>Account</th><th class="number">
                    Amount
                  </th><th><span class="sr-only">Actions</span></th>
                </tr>
              </thead><tbody>
                {#each transactions as transaction (transaction.id)}<tr>
                    <td class="entry-date">{dateLabel(transaction.date)}</td><td
                      class="entry-description"
                    >
                      {transaction.description}
                    </td><td class="entry-account">{transaction.account_name}</td><td
                      class:income={BigInt(transaction.amount_cents) > 0n}
                      class="number entry-amount"
                    >
                      {money(transaction.amount_cents, true)}
                    </td><td class="entry-action">
                      <button
                        class="button icon-button"
                        aria-label={`Edit ${transaction.description}`}
                        onclick={() => openEntry(transaction)}
                      >
                        Edit
                      </button>
                    </td>
                  </tr>{/each}
              </tbody>
            </table>
          {/if}
        </section>
        <aside class="accounts-panel" aria-labelledby="accounts-heading">
          <div class="section-heading">
            <h2 id="accounts-heading">Your accounts</h2>
            <button class="button quiet" onclick={openAccount}>+ Add</button>
          </div>
          <p class="muted account-explainer">Recorded balances · all dates</p>
          {#each accounts as account (account.id)}
            <section class="account-item" aria-label={`${account.name} account`}>
              <div class="account-row">
                <h3>{account.name}</h3>
                <span class="account-monogram" aria-hidden="true">
                  {account.name.slice(0, 1).toUpperCase()}
                </span>
              </div>
              <p class="account-balance">{money(account.balance_cents)}</p>
              <p class="account-date">
                {account.latest_transaction_date
                  ? `Latest entry: ${dateLabel(account.latest_transaction_date)}`
                  : `Opening: ${dateLabel(account.opening_date)}`}
              </p>
              <p class="account-caveat">Complete history isn't verified.</p>
            </section>
          {:else}<p class="muted no-accounts">No accounts yet.</p>{/each}
          <p class="aside-footnote">
            Opening balances set the starting point. They don't count as income.
          </p>
        </aside>
      </div>
      <footer>coinz · Small steps. Fewer financial mysteries.</footer>
    </main>
  </div>
{/if}

<dialog
  bind:this={accountDialog}
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  aria-labelledby="account-title"
>
  <form onsubmit={saveAccount}>
    <div class="dialog-heading">
      <div>
        <p class="eyebrow">A STARTING POINT</p>
        <h2 id="account-title">Add an account</h2>
      </div>
      <button
        type="button"
        class="button quiet"
        onclick={() => accountDialog.close()}
        disabled={busy}
        aria-label="Close add account"
      >
        Close
      </button>
    </div>
    <label>Account name<input name="name" placeholder="e.g. N26" required maxlength="80" /></label>
    <label>
      Opening date<input
        name="opening_date"
        type="date"
        value={localDate()}
        min="0001-01-01"
        max="9999-12-31"
        required
      />
    </label>
    <label>
      Opening balance (€)<input
        name="opening_balance"
        inputmode="decimal"
        value="0.00"
        required
        aria-describedby="opening-help"
      />
    </label>
    <p class="form-help" id="opening-help">
      The balance at the start of this date, before any transactions you'll enter for it. Negative
      balances are allowed.
    </p>
    {#if formError}<p class="error" role="alert">{formError}</p>{/if}
    <button class="button primary full" disabled={busy}>
      {busy ? 'Saving…' : 'Create account'}
    </button>
  </form>
</dialog>

<dialog
  bind:this={transactionDialog}
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  aria-labelledby="transaction-title"
>
  <form onsubmit={saveEntry}>
    <div class="dialog-heading">
      <div>
        <p class="eyebrow">IN THE NOTEBOOK</p>
        <h2 id="transaction-title">{editing ? 'Edit transaction' : 'Add a transaction'}</h2>
      </div>
      <button
        type="button"
        class="button quiet"
        onclick={() => transactionDialog.close()}
        disabled={busy}
        aria-label="Close transaction"
      >
        Close
      </button>
    </div>
    <div class="kind-selector" aria-label="Transaction type">
      <label class:chosen={entryKind === 'expense'}>
        <input type="radio" name="kind" value="expense" bind:group={entryKind} />Expense
      </label><label class:chosen={entryKind === 'income'}>
        <input type="radio" name="kind" value="income" bind:group={entryKind} />Income
      </label>
    </div>
    <label>
      Amount (€)<input bind:value={entryAmount} inputmode="decimal" placeholder="0.00" required />
    </label>
    <label>
      Description<input
        bind:value={entryDescription}
        placeholder="What was it for?"
        required
        maxlength="200"
      />
    </label>
    <div class="form-row">
      <label>
        Date<input
          type="date"
          bind:value={entryDate}
          min={accounts.find((account) => account.id === entryAccount)?.opening_date ??
            '0001-01-01'}
          max="9999-12-31"
          required
        />
      </label><label>
        Account<select bind:value={entryAccount} required>
          {#each accounts as account (account.id)}<option value={account.id}>
              {account.name}
            </option>{/each}
        </select>
      </label>
    </div>
    {#if formError}<p class="error" role="alert">{formError}</p>{/if}
    <button class="button primary full" disabled={busy || deleting}>
      {busy ? 'Saving…' : editing ? 'Save changes' : 'Record transaction'}
    </button>
    {#if editing}<div class="delete-area">
        {#if deleting}<p>Delete this transaction? Its amount will be removed from your records.</p>
          <div class="delete-actions">
            <button type="button" class="button" onclick={() => (deleting = false)} disabled={busy}>
              Keep it
            </button><button
              type="button"
              class="button destructive"
              onclick={deleteEntry}
              disabled={busy}
            >
              Delete transaction
            </button>
          </div>{:else}<button
            type="button"
            class="button quiet danger-text"
            onclick={() => (deleting = true)}
            disabled={busy}
          >
            Delete transaction
          </button>{/if}
      </div>{/if}
  </form>
</dialog>
