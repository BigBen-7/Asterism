import { describe, expect, it } from "vitest";
import { last } from "../src/tasks/038-last";

// Remove `.skip` once you start the task.
describe.skip("last", () => {
  it("works", () => {
    expect(last([1, 2, 3], 2)).toEqual([2, 3]);
    expect(last([1, 2, 3], 0)).toEqual([]);
  });
});
