import { describe, expect, it } from "vitest";
import { mergeSorted } from "../src/tasks/091-merge-sorted";

// Remove `.skip` once you start the task.
describe.skip("mergeSorted", () => {
  it("works", () => {
    expect(mergeSorted([1, 4], [2, 3, 5])).toEqual([1, 2, 3, 4, 5]);
    expect(mergeSorted([], [1])).toEqual([1]);
  });
});
