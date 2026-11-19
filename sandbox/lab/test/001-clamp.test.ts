import { describe, expect, it } from "vitest";
import { clamp } from "../src/tasks/001-clamp";

// Remove `.skip` once you start the task.
describe.skip("clamp", () => {
  it("works", () => {
    expect(clamp(5, 0, 10)).toEqual(5);
    expect(clamp(-1, 0, 10)).toEqual(0);
    expect(clamp(11, 0, 10)).toEqual(10);
  });
});
