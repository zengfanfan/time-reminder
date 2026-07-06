import assert from "node:assert/strict";
import { mock, test } from "node:test";
import { get } from "svelte/store";

interface InvokeCall {
  command: string;
  payload: unknown;
}

const calls: InvokeCall[] = [];
const storage = new Map<string, string>();

globalThis.localStorage = {
  get length() {
    return storage.size;
  },
  clear: () => storage.clear(),
  getItem: (key: string) => storage.get(key) ?? null,
  key: (index: number) => Array.from(storage.keys())[index] ?? null,
  removeItem: (key: string) => storage.delete(key),
  setItem: (key: string, value: string) => {
    storage.set(key, String(value));
  },
};

Object.defineProperty(globalThis, "navigator", {
  configurable: true,
  value: { language: "zh-CN", languages: ["zh-CN", "en-US"] },
});

mock.module("@tauri-apps/api/core", {
  exports: {
    invoke: async (command: string, payload?: unknown) => {
      calls.push({ command, payload });
    },
  },
});

const { initLocale, locale, t, toggleLocale } = await import("../../src/lib/i18n.ts");

test("integration_initLocale prefers saved locale and syncs it to Tauri", () => {
  calls.length = 0;
  storage.set("time-reminder-locale", "en");

  initLocale();

  assert.equal(get(locale), "en");
  assert.equal(get(t).addReminder, "Add Reminder");
  assert.deepEqual(calls.pop(), {
    command: "set_locale",
    payload: { locale: "en" },
  });
});

test("integration_initLocale falls back to system language", () => {
  calls.length = 0;
  storage.clear();

  initLocale();

  assert.equal(get(locale), "zh");
  assert.deepEqual(calls.pop(), {
    command: "set_locale",
    payload: { locale: "zh" },
  });
});

test("integration_toggleLocale persists and syncs next locale", () => {
  calls.length = 0;
  storage.set("time-reminder-locale", "zh");
  initLocale();
  calls.length = 0;

  toggleLocale();

  assert.equal(get(locale), "en");
  assert.equal(storage.get("time-reminder-locale"), "en");
  assert.deepEqual(calls.pop(), {
    command: "set_locale",
    payload: { locale: "en" },
  });
});
