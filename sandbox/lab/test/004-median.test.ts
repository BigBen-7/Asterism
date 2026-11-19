import { describe, expect, it } from "vitest";
import { median } from "../src/tasks/004-median";

// Remove `.skip` once you start the task.
describe.skip("median", () => {
  it("works", () => {
    expect(median([3, 1, 2])).toEqual(2);
    expect(median([1, 2, 3, 4])).toEqual(2.5);
    expect(median([])).toEqual(0);
  });
});
