import { describe, expect, it } from "vitest";
import { invert } from "../src/tasks/051-invert";

// Remove `.skip` once you start the task.
describe.skip("invert", () => {
  it("works", () => {
    expect(invert({ a: 'x', b: 'y' })).toEqual({ x: 'a', y: 'b' });
  });
});
