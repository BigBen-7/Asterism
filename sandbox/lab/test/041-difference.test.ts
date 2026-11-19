import { describe, expect, it } from "vitest";
import { difference } from "../src/tasks/041-difference";

// Remove `.skip` once you start the task.
describe.skip("difference", () => {
  it("works", () => {
    expect(difference([1, 2, 3], [2])).toEqual([1, 3]);
  });
});
