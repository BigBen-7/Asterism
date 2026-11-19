import { describe, expect, it } from "vitest";
import { roundTo } from "../src/tasks/068-round-to";

// Remove `.skip` once you start the task.
describe.skip("roundTo", () => {
  it("works", () => {
    expect(roundTo(3.14159, 2)).toEqual(3.14);
    expect(roundTo(2.5, 0)).toEqual(3);
  });
});
