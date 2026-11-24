/** Only the final chart dimension becomes a Number; monetary arithmetic stays integer. */
export function barPercent(cents: string, maximum: bigint): number {
  return maximum === 0n ? 0 : Number((BigInt(cents) * 10000n) / maximum) / 100;
}

export function monthStatus(month: string, today: string): string {
  const current = today.slice(0, 7);
  return month === current ? 'In progress' : month > current ? 'Future' : '';
}
