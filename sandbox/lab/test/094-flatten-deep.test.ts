import { describe, expect, it } from "vitest";
import { flattenDeep } from "../src/tasks/094-flatten-deep";

// Remove `.skip` once you start the task.
describe.skip("flattenDeep", () => {
  it("works", () => {
    expect(flattenDeep([1, [2, [3, [4]]]])).toEqual([1, 2, 3, 4]);
    expect(flattenDeep([])).toEqual([]);
  });
});
