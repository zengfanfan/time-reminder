import assert from "node:assert/strict";
import { mock, test } from "node:test";
import type { ReminderConfig } from "../../src/lib/types.ts";

interface InvokeCall {
  command: string;
  payload: unknown;
}

const calls: InvokeCall[] = [];

mock.module("@tauri-apps/api/core", {
  exports: {
    invoke: async (command: string, payload?: unknown) => {
      calls.push({ command, payload });
      return { command, payload };
    },
  },
});

const {
  deleteReminder,
  getCountdowns,
  loadReminders,
  saveReminder,
  toggleReminder,
} = await import("../../src/lib/reminders.ts");

test("integration_reminder_command_wrappers call the expected Tauri commands", async (t) => {
  t.beforeEach(() => {
    calls.length = 0;
  });

  const config: ReminderConfig = {
    id: "r1",
    name: "Break",
    text: "Move",
    interval_secs: 60,
    display_secs: 10,
    enabled: true,
    play_sound: false,
    fullscreen: true,
  };

  assert.deepEqual(await loadReminders(), {
    command: "get_reminders",
    payload: undefined,
  });
  assert.deepEqual(calls.pop(), { command: "get_reminders", payload: undefined });

  assert.deepEqual(await getCountdowns(), {
    command: "get_countdowns",
    payload: undefined,
  });
  assert.deepEqual(calls.pop(), { command: "get_countdowns", payload: undefined });

  assert.deepEqual(await saveReminder(config), {
    command: "save_reminder",
    payload: { config },
  });
  assert.deepEqual(calls.pop(), {
    command: "save_reminder",
    payload: { config },
  });

  assert.deepEqual(await deleteReminder("r1"), {
    command: "delete_reminder",
    payload: { id: "r1" },
  });
  assert.deepEqual(calls.pop(), {
    command: "delete_reminder",
    payload: { id: "r1" },
  });

  assert.deepEqual(await toggleReminder("r1", false), {
    command: "toggle_reminder",
    payload: { id: "r1", enabled: false },
  });
  assert.deepEqual(calls.pop(), {
    command: "toggle_reminder",
    payload: { id: "r1", enabled: false },
  });
});
