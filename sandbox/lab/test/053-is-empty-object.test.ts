import { describe, expect, it } from "vitest";
import { isEmptyObject } from "../src/tasks/053-is-empty-object";

// Remove `.skip` once you start the task.
describe.skip("isEmptyObject", () => {
  it("works", () => {
    expect(isEmptyObject({})).toEqual(true);
    expect(isEmptyObject({ a: 1 })).toEqual(false);
  });
});
