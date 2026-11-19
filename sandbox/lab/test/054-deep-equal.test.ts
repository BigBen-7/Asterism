import { describe, expect, it } from "vitest";
import { deepEqual } from "../src/tasks/054-deep-equal";

// Remove `.skip` once you start the task.
describe.skip("deepEqual", () => {
  it("works", () => {
    expect(deepEqual({ a: [1, { b: 2 }] }, { a: [1, { b: 2 }] })).toEqual(true);
    expect(deepEqual({ a: 1, b: 2 }, { b: 2, a: 1 })).toEqual(true);
    expect(deepEqual({ a: 1 }, { a: 2 })).toEqual(false);
    expect(deepEqual([1, 2], [1, 2, 3])).toEqual(false);
  });
});
