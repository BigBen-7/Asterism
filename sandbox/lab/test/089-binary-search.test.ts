import { describe, expect, it } from "vitest";
import { binarySearch } from "../src/tasks/089-binary-search";

// Remove `.skip` once you start the task.
describe.skip("binarySearch", () => {
  it("works", () => {
    expect(binarySearch([1, 3, 5, 7], 5)).toEqual(2);
    expect(binarySearch([1, 3, 5, 7], 4)).toEqual(-1);
    expect(binarySearch([], 1)).toEqual(-1);
  });
});
