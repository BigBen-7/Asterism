import { describe, expect, it } from "vitest";
import { flatten } from "../src/tasks/034-flatten";

// Remove `.skip` once you start the task.
describe.skip("flatten", () => {
  it("works", () => {
    expect(flatten([[1, 2], [3], []])).toEqual([1, 2, 3]);
  });
});
