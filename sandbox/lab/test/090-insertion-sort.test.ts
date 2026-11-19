import { describe, expect, it } from "vitest";
import { insertionSort } from "../src/tasks/090-insertion-sort";

// Remove `.skip` once you start the task.
describe.skip("insertionSort", () => {
  it("works", () => {
    expect(insertionSort([3, 1, 2])).toEqual([1, 2, 3]);
    expect(insertionSort([])).toEqual([]);
  });
});
