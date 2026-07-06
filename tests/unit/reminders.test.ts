import assert from "node:assert/strict";
import { test } from "node:test";

import { createDefaultReminder } from "../../src/lib/reminders.ts";

test("unit_createDefaultReminder uses translated defaults when provided", () => {
  const reminder = createDefaultReminder({
    defaultName: "Stretch",
    defaultText: "Stand up",
  });

  assert.equal(reminder.name, "Stretch");
  assert.equal(reminder.text, "Stand up");
  assert.equal(reminder.interval_secs, 1800);
  assert.equal(reminder.display_secs, 300);
  assert.equal(reminder.enabled, true);
  assert.equal(reminder.play_sound, true);
  assert.equal(reminder.fullscreen, false);
  assert.equal(typeof reminder.id, "string");
  assert.ok(reminder.id.length > 0);
});

test("unit_createDefaultReminder falls back to English defaults", () => {
  const reminder = createDefaultReminder();

  assert.equal(reminder.name, "New Reminder");
  assert.match(reminder.text, /^Time for a break!/);
});
