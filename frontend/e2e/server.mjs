import { spawn, spawnSync } from 'node:child_process';
import { closeSync, mkdtempSync, openSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';

const root = resolve('..');
const binary = join(root, 'target/release/coinz');
const data = mkdtempSync(join(tmpdir(), 'coinz-e2e-'));
// Keep the synthetic fixture bounded on hosts reporting a large CPU count.
const testEnv = { ...process.env, TOKIO_WORKER_THREADS: '2' };
const passwordFile = join(data, 'synthetic-password');
writeFileSync(passwordFile, 'synthetic-browser-password\n', { mode: 0o600 });
for (const username of [
  'desktop-user',
  'phone-user',
  'desktop-settings-source',
  'phone-settings-source',
  'desktop-settings-restore',
  'phone-settings-restore',
]) {
  const passwordInput = openSync(passwordFile, 'r');
  const result = spawnSync(
    binary,
    ['--data-dir', data, 'create-user', username, '--password-stdin'],
    {
      stdio: [passwordInput, 'pipe', 'pipe'],
      encoding: 'utf8',
      timeout: 10_000,
      env: testEnv,
    },
  );
  closeSync(passwordInput);
  if (result.status !== 0) {
    rmSync(data, { recursive: true, force: true });
    throw new Error(
      result.stderr || result.error?.message || 'Could not create browser-test user.',
    );
  }
}
const child = spawn(binary, ['serve'], {
  cwd: root,
  stdio: 'inherit',
  env: {
    ...testEnv,
    COINZ_DATA_DIR: data,
    COINZ_BIND: '127.0.0.1:4100',
    COINZ_ORIGIN: 'http://127.0.0.1:4100',
    COINZ_STATIC_DIR: join(root, 'frontend/build'),
  },
});
let stopping = false;
for (const signal of ['SIGINT', 'SIGTERM'])
  process.on(signal, () => {
    stopping = true;
    child.kill('SIGINT');
  });
child.on('exit', (code) => {
  rmSync(data, { recursive: true, force: true });
  process.exit(stopping ? 0 : (code ?? 1));
});
