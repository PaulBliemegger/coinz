/** Delegate Escape from native controls without changing their semantic roles. */
export function escapeToCancel(node: HTMLElement, cancel: () => void) {
  const handle = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      cancel();
    }
  };
  node.addEventListener('keydown', handle);
  return { destroy: () => node.removeEventListener('keydown', handle) };
}
