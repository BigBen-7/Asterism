import { describe, expect, it } from "vitest";
import { sortNumbers } from "../src/tasks/046-sort-numbers";

// Remove `.skip` once you start the task.
describe.skip("sortNumbers", () => {
  it("works", () => {
    expect(sortNumbers([10, 2, 33, 4])).toEqual([2, 4, 10, 33]);
  });
});
