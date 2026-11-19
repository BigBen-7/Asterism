import { describe, expect, it } from "vitest";
import { isEven } from "../src/tasks/007-is-even";

// Remove `.skip` once you start the task.
describe.skip("isEven", () => {
  it("works", () => {
    expect(isEven(4)).toEqual(true);
    expect(isEven(7)).toEqual(false);
    expect(isEven(0)).toEqual(true);
  });
});
