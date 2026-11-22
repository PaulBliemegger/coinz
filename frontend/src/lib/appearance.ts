export type Appearance = 'system' | 'light' | 'dark';
export const appearanceKey = 'coinz.appearance';

export function parseAppearance(value: unknown): Appearance {
  return value === 'light' || value === 'dark' ? value : 'system';
}

export function setAppearance(value: Appearance) {
  document.documentElement.dataset.appearance = value;
  try {
    localStorage.setItem(appearanceKey, value);
  } catch {
    /* Apply even without storage. */
  }
}
