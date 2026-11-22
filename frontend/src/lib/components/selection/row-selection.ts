import type { Action } from 'svelte/action';

type Selection = { selected: boolean; disabled: boolean; onselect: (checked: boolean) => void };

/** Row whitespace is a pointer shortcut; the native checkbox remains the keyboard control. */
export const rowSelection: Action<HTMLElement, Selection> = (node, initial) => {
  let selection = initial;
  function click(event: MouseEvent) {
    if (selection.disabled || event.defaultPrevented || event.button !== 0) return;
    const target = event.target;
    if (
      !(target instanceof Element) ||
      target.closest(
        'button,input,select,textarea,label,form,a,summary,[role="button"],[contenteditable]',
      )
    )
      return;
    if (window.getSelection()?.isCollapsed === false) return;
    selection.onselect(!selection.selected);
  }
  node.addEventListener('click', click);
  return {
    update(next) {
      selection = next;
    },
    destroy() {
      node.removeEventListener('click', click);
    },
  };
};
