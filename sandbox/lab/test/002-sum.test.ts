import { describe, expect, it } from "vitest";
import { sum } from "../src/tasks/002-sum";

// Remove `.skip` once you start the task.
describe.skip("sum", () => {
  it("works", () => {
    expect(sum([1, 2, 3])).toEqual(6);
    expect(sum([])).toEqual(0);
  });
});
