import assert from "node:assert/strict";
import { test } from "node:test";

import { formatCountdownLocale, formatDurationLocale } from "../../src/lib/i18n.ts";
import type { Translations } from "../../src/lib/types.ts";

const tr: Pick<Translations, "durationH" | "durationM" | "durationS"> = {
  durationH: (h, m) => (m > 0 ? `${h}h ${m}m` : `${h}h`),
  durationM: (m, s) => (s > 0 ? `${m}m ${s}s` : `${m}m`),
  durationS: (s) => `${s}s`,
};

test("unit_formatDurationLocale formats seconds, minutes, and hours", () => {
  assert.equal(formatDurationLocale(45, tr), "45s");
  assert.equal(formatDurationLocale(120, tr), "2m");
  assert.equal(formatDurationLocale(125, tr), "2m 5s");
  assert.equal(formatDurationLocale(3600, tr), "1h");
  assert.equal(formatDurationLocale(3660, tr), "1h 1m");
});

test("unit_formatCountdownLocale pads sub-hour countdowns and hides long values", () => {
  assert.equal(formatCountdownLocale(0, tr), "00:00");
  assert.equal(formatCountdownLocale(9, tr), "00:09");
  assert.equal(formatCountdownLocale(61, tr), "01:01");
  assert.equal(formatCountdownLocale(-5, tr), "00:00");
  assert.equal(formatCountdownLocale(3600, tr), null);
});
