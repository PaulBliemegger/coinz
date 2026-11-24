/** Only the final chart dimension becomes a Number; monetary arithmetic stays integer. */
export function barPercent(cents: string, maximum: bigint): number {
  return maximum === 0n ? 0 : Number((BigInt(cents) * 10000n) / maximum) / 100;
}

export function monthStatus(month: string, today: string): string {
  const current = today.slice(0, 7);
  return month === current ? 'In progress' : month > current ? 'Future' : '';
}

/** Aim for five readable grid intervals, with €50 as the minimum step. */
export function chartScale(maximum: bigint): { top: bigint; ticks: bigint[] } {
  let step = 5000n;
  const multipliers = [1n, 2n, 5n];
  let magnitude = 5000n;
  while (true) {
    const candidate = multipliers.map((n) => n * magnitude).find((n) => n * 5n >= maximum);
    if (candidate !== undefined) {
      step = candidate;
      break;
    }
    magnitude *= 10n;
  }
  const top = ((maximum + step - 1n) / step || 1n) * step;
  return { top, ticks: Array.from({ length: Number(top / step) + 1 }, (_, i) => BigInt(i) * step) };
}
