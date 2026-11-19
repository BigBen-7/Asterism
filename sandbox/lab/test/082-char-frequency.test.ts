import { describe, expect, it } from "vitest";
import { charFrequency } from "../src/tasks/082-char-frequency";

// Remove `.skip` once you start the task.
describe.skip("charFrequency", () => {
  it("works", () => {
    expect(charFrequency('aab')).toEqual({ a: 2, b: 1 });
    expect(charFrequency('')).toEqual({});
  });
});
