import { describe, expect, it } from "vitest";
import { deepClone } from "../src/tasks/055-deep-clone";

// Remove `.skip` once you start the task.
describe.skip("deepClone", () => {
  it("works", () => {
    const src = { a: [1, { b: 2 }], c: 'x' };
    const copy = deepClone(src) as typeof src;
    expect(copy).toEqual(src);
    expect(copy).not.toBe(src);
    expect(copy.a).not.toBe(src.a);
    expect(copy.a[1]).not.toBe(src.a[1]);
  });
});
