import { describe, expect, it } from "vitest";
import { first } from "../src/tasks/037-first";

// Remove `.skip` once you start the task.
describe.skip("first", () => {
  it("works", () => {
    expect(first([1, 2, 3], 2)).toEqual([1, 2]);
    expect(first([1], 3)).toEqual([1]);
  });
});
