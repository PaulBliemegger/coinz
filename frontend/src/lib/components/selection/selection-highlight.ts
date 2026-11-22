import type { Action } from 'svelte/action';

/** Fade a separate wrapper so opacity cannot flatten the background's 3D faces. */
export const selectionHighlight: Action<HTMLElement, boolean> = (layer, initial) => {
  const flip = layer.querySelector<HTMLElement>('[data-selection-flip]');
  if (!flip) return;
  const motion = window.matchMedia('(prefers-reduced-motion: reduce)');
  let selected = initial;
  let generation = 0;
  let animations: Animation[] = [];
  layer.style.opacity = selected ? '1' : '0';

  function settle() {
    generation += 1;
    for (const animation of animations) animation.cancel();
    animations = [];
    if (flip) flip.style.transform = '';
  }
  function motionChanged() {
    if (motion.matches) settle();
  }
  motion.addEventListener('change', motionChanged);

  return {
    update(next) {
      if (next === selected) return;
      const from = {
        opacity: getComputedStyle(layer).opacity,
        transform: getComputedStyle(flip).transform,
      };
      const interrupted = animations.some((animation) => animation.playState === 'running');
      settle();
      selected = next;
      layer.style.opacity = selected ? '1' : '0';
      if (motion.matches) return;

      if (selected) {
        animations.push(
          flip.animate(
            [
              { transform: interrupted ? from.transform : 'rotateX(0deg)' },
              { transform: 'rotateX(180deg)' },
            ],
            { duration: 280, easing: 'cubic-bezier(.2,.7,.2,1)' },
          ),
        );
        if (interrupted)
          animations.push(
            layer.animate([{ opacity: from.opacity }, { opacity: 1 }], {
              duration: 120,
              easing: 'ease-out',
            }),
          );
      } else {
        // Mid-flip deselection freezes the current angle rather than snapping it flat.
        flip.style.transform = from.transform;
        const version = generation;
        const fade = layer.animate([{ opacity: from.opacity }, { opacity: 0 }], {
          duration: 160,
          easing: 'ease-out',
        });
        fade.onfinish = () => {
          if (version === generation && !selected) flip.style.transform = '';
        };
        animations.push(fade);
      }
    },
    destroy() {
      settle();
      motion.removeEventListener('change', motionChanged);
    },
  };
};
