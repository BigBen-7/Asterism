import { describe, expect, it } from "vitest";
import { isOdd } from "../src/tasks/008-is-odd";

// Remove `.skip` once you start the task.
describe.skip("isOdd", () => {
  it("works", () => {
    expect(isOdd(3)).toEqual(true);
    expect(isOdd(0)).toEqual(false);
    expect(isOdd(-3)).toEqual(true);
  });
});
