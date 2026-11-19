import { describe, expect, it } from "vitest";
import { isBlank } from "../src/tasks/030-is-blank";

// Remove `.skip` once you start the task.
describe.skip("isBlank", () => {
  it("works", () => {
    expect(isBlank('')).toEqual(true);
    expect(isBlank('   ')).toEqual(true);
    expect(isBlank(' a ')).toEqual(false);
  });
});
