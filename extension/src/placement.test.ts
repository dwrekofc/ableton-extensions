import assert from "node:assert/strict";
import test from "node:test";
import { sdkInsertionIndex } from "./placement.js";

test("official SDK supports deterministic chain boundaries", () => {
  assert.equal(sdkInsertionIndex("beginning", 3), 0);
  assert.equal(sdkInsertionIndex("end", 3), 3);
});

test("official SDK rejects selected-device placement", () => {
  assert.throws(() => sdkInsertionIndex("before_selected", 3), /does not expose/);
  assert.throws(() => sdkInsertionIndex("after_selected", 3), /does not expose/);
});
