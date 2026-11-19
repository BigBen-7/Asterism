import { describe, expect, it } from "vitest";
import { average } from "../src/tasks/003-average";

// Remove `.skip` once you start the task.
describe.skip("average", () => {
  it("works", () => {
    expect(average([2, 4, 6])).toEqual(4);
    expect(average([])).toEqual(0);
  });
});
