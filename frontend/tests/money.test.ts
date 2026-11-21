import { test } from 'node:test';
import assert from 'node:assert/strict';
import { amountInput, localDate, money } from '../src/lib/money.ts';

test('formats cents exactly, including negative fractions and i64 boundaries', () => {
  assert.equal(money('1'), '€0.01');
  assert.equal(money('-1'), '−€0.01');
  assert.equal(money('298766'), '€2,987.66');
  assert.equal(money('9223372036854775807'), '€92,233,720,368,547,758.07');
  assert.equal(money('-9223372036854775808'), '−€92,233,720,368,547,758.08');
  assert.equal(money('1234', true), '+€12.34');
});

test('editing amounts preserves their cents without floating point', () => {
  assert.equal(amountInput('-1234'), '12.34');
  assert.equal(amountInput('1'), '0.01');
  assert.equal(amountInput('9223372036854775807'), '92233720368547758.07');
});

test('entry defaults use the local calendar date, including near midnight', () => {
  const localMidnight = new Date(2026, 10, 1, 0, 5);
  assert.equal(localDate(localMidnight), '2026-11-01');
});
