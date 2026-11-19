import { describe, expect, it } from "vitest";
import { sumMatrix } from "../src/tasks/093-sum-matrix";

// Remove `.skip` once you start the task.
describe.skip("sumMatrix", () => {
  it("works", () => {
    expect(sumMatrix([[1, 2], [3, 4]])).toEqual(10);
    expect(sumMatrix([])).toEqual(0);
  });
});
