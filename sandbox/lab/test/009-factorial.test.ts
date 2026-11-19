import { describe, expect, it } from "vitest";
import { factorial } from "../src/tasks/009-factorial";

// Remove `.skip` once you start the task.
describe.skip("factorial", () => {
  it("works", () => {
    expect(factorial(0)).toEqual(1);
    expect(factorial(5)).toEqual(120);
  });
});
