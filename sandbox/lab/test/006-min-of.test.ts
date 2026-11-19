import { describe, expect, it } from "vitest";
import { minOf } from "../src/tasks/006-min-of";

// Remove `.skip` once you start the task.
describe.skip("minOf", () => {
  it("works", () => {
    expect(minOf([4, 2, 8])).toEqual(2);
    expect(minOf([])).toEqual(undefined);
  });
});
