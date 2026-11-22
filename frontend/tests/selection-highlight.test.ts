import { test } from 'node:test';
import assert from 'node:assert/strict';
import { selectionHighlight } from '../src/lib/components/selection/selection-highlight.ts';

function fixture(t: { after: (fn: () => void) => void }, selected = false) {
  const descriptors = new Map(
    ['window', 'getComputedStyle'].map((key) => [
      key,
      Object.getOwnPropertyDescriptor(globalThis, key),
    ]),
  );
  t.after(() => {
    for (const [key, descriptor] of descriptors) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  });
  const listeners = new Set<() => void>();
  const motion = {
    matches: false,
    addEventListener: (_: string, listener: () => void) => listeners.add(listener),
    removeEventListener: (_: string, listener: () => void) => listeners.delete(listener),
  };
  const makeLayer = () => ({
    style: { opacity: '', transform: '' },
    visual: { opacity: '1', transform: 'rotateX(180deg)' },
    calls: [] as {
      frames: Keyframe[];
      options: KeyframeAnimationOptions;
      animation: { playState: string; onfinish: (() => void) | null; cancel: () => void };
    }[],
    animate(frames: Keyframe[], options: KeyframeAnimationOptions) {
      const animation = {
        playState: 'running',
        onfinish: null as (() => void) | null,
        cancel() {
          this.playState = 'idle';
        },
      };
      this.calls.push({ frames, options, animation });
      return animation as unknown as Animation;
    },
  });
  const flip = makeLayer();
  const layer = { ...makeLayer(), querySelector: () => flip };
  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: { matchMedia: () => motion },
  });
  Object.defineProperty(globalThis, 'getComputedStyle', {
    configurable: true,
    value: (node: typeof layer | typeof flip) => node.visual,
  });
  const action = selectionHighlight(layer as unknown as HTMLElement, selected);
  assert.ok(action?.update && action.destroy);
  const update = action.update;
  const destroy = action.destroy;
  return { layer, flip, motion, listeners, update, destroy };
}

test('selection flips behind stationary content; deselection only fades the separate wrapper', (t) => {
  const f = fixture(t);
  assert.equal(f.layer.style.opacity, '0');
  assert.equal(f.flip.calls.length, 0, 'initial rendering must not animate');
  f.update(true);
  assert.equal(f.layer.style.opacity, '1');
  assert.equal(f.flip.calls.length, 1);
  assert.equal(f.flip.calls[0].options.duration, 280);
  assert(f.flip.calls[0].frames.every((frame) => !('opacity' in frame)));
  f.flip.calls[0].animation.playState = 'finished';
  f.update(false);
  assert.equal(f.flip.calls.length, 1, 'deselection must not run a reverse flip');
  assert(f.layer.calls[0].frames.every((frame) => !('transform' in frame)));
  assert.equal(f.layer.calls[0].options.duration, 160);
  assert.equal(f.layer.style.opacity, '0');
  f.layer.calls[0].animation.onfinish?.();
  assert.equal(f.flip.style.transform, '');
  f.destroy();
  assert.equal(f.listeners.size, 0);
});

test('rapid toggles continue from the visible state and stale completion cannot reset a later fade', (t) => {
  const f = fixture(t);
  f.update(true);
  f.flip.visual.transform = 'rotateX(110deg)';
  f.update(false);
  assert.equal(f.flip.calls[0].animation.playState, 'idle');
  assert.equal(f.flip.style.transform, 'rotateX(110deg)');
  const oldCompletion = f.layer.calls[0].animation.onfinish;
  f.layer.visual.opacity = '0.4';
  f.update(true);
  assert.equal(f.layer.calls[0].animation.playState, 'idle');
  assert.equal(f.flip.calls[1].frames[0].transform, 'rotateX(110deg)');
  assert.equal(f.layer.calls[1].frames[0].opacity, '0.4');
  f.flip.visual.transform = 'rotateX(155deg)';
  f.update(false);
  oldCompletion?.();
  assert.equal(
    f.flip.style.transform,
    'rotateX(155deg)',
    'a previous fade must not alter the new one',
  );
  f.destroy();
  assert(f.flip.calls.every((call) => call.animation.playState === 'idle'));
  assert(f.layer.calls.every((call) => call.animation.playState === 'idle'));
});

test('enabling reduced motion stops in-flight effects and subsequent choices settle immediately', (t) => {
  const f = fixture(t);
  f.update(true);
  f.motion.matches = true;
  for (const listener of f.listeners) listener();
  assert.equal(f.flip.calls[0].animation.playState, 'idle');
  f.update(false);
  f.update(true);
  assert.equal(f.layer.style.opacity, '1');
  assert.equal(f.flip.calls.length, 1);
  assert.equal(f.layer.calls.length, 0);
  f.destroy();
});
