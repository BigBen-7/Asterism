import { describe, expect, it } from "vitest";
import { lcm } from "../src/tasks/013-lcm";

// Remove `.skip` once you start the task.
describe.skip("lcm", () => {
  it("works", () => {
    expect(lcm(4, 6)).toEqual(12);
    expect(lcm(3, 5)).toEqual(15);
  });
});
