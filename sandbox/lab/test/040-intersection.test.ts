import { describe, expect, it } from "vitest";
import { intersection } from "../src/tasks/040-intersection";

// Remove `.skip` once you start the task.
describe.skip("intersection", () => {
  it("works", () => {
    expect(intersection([1, 2, 3], [2, 3, 4])).toEqual([2, 3]);
  });
});
