import { describe, expect, it } from "vitest";
import { omit } from "../src/tasks/050-omit";

// Remove `.skip` once you start the task.
describe.skip("omit", () => {
  it("works", () => {
    expect(omit({ a: 1, b: 2 }, ['a'])).toEqual({ b: 2 });
  });
});
