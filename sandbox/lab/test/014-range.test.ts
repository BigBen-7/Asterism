import { describe, expect, it } from "vitest";
import { range } from "../src/tasks/014-range";

// Remove `.skip` once you start the task.
describe.skip("range", () => {
  it("works", () => {
    expect(range(0, 3)).toEqual([0, 1, 2]);
    expect(range(2, 2)).toEqual([]);
  });
});
