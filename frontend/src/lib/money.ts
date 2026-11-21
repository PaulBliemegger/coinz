const euro = new Intl.NumberFormat('en-IE', { style: 'currency', currency: 'EUR' });

/** Cents travel as decimal strings; BigInt preserves every cent of an i64. */
export function money(value: string, showPlus = false): string {
  const cents = BigInt(value);
  const absolute = cents < 0n ? -cents : cents;
  const fraction = (absolute % 100n).toString().padStart(2, '0');
  const formatted = euro
    .formatToParts(absolute / 100n)
    .map((part) => (part.type === 'fraction' ? fraction : part.value))
    .join('');
  return `${cents < 0n ? '−' : showPlus && cents > 0n ? '+' : ''}${formatted}`;
}

export function amountInput(value: string): string {
  const cents = BigInt(value);
  const absolute = cents < 0n ? -cents : cents;
  return `${absolute / 100n}.${(absolute % 100n).toString().padStart(2, '0')}`;
}

export function localDate(now = new Date()): string {
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
}

export function dateLabel(date: string): string {
  return new Intl.DateTimeFormat('en-GB', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  }).format(new Date(`${date}T12:00:00`));
}

export function monthLabel(month: string): string {
  return new Intl.DateTimeFormat('en-GB', { month: 'long', year: 'numeric' }).format(
    new Date(`${month}-01T12:00:00`),
  );
}
