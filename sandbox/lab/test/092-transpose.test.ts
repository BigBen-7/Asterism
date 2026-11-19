import { describe, expect, it } from "vitest";
import { transpose } from "../src/tasks/092-transpose";

// Remove `.skip` once you start the task.
describe.skip("transpose", () => {
  it("works", () => {
    expect(transpose([[1, 2, 3], [4, 5, 6]])).toEqual([[1, 4], [2, 5], [3, 6]]);
  });
});
