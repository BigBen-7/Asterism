import { describe, expect, it } from "vitest";
import { fibonacci } from "../src/tasks/010-fibonacci";

// Remove `.skip` once you start the task.
describe.skip("fibonacci", () => {
  it("works", () => {
    expect(fibonacci(0)).toEqual(0);
    expect(fibonacci(1)).toEqual(1);
    expect(fibonacci(10)).toEqual(55);
  });
});
