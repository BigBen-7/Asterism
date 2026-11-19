import { describe, expect, it } from "vitest";
import { maxOf } from "../src/tasks/005-max-of";

// Remove `.skip` once you start the task.
describe.skip("maxOf", () => {
  it("works", () => {
    expect(maxOf([1, 9, 3])).toEqual(9);
    expect(maxOf([])).toEqual(undefined);
  });
});
