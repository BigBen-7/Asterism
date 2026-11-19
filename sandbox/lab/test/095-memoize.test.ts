import { describe, expect, it } from "vitest";
import { memoize } from "../src/tasks/095-memoize";

// Remove `.skip` once you start the task.
describe.skip("memoize", () => {
  it("works", () => {
    let calls = 0;
    const slow = (n: number) => {
      calls += 1;
      return n * 2;
    };
    const fast = memoize(slow);
    expect(fast(2)).toBe(4);
    expect(fast(2)).toBe(4);
    expect(fast(3)).toBe(6);
    expect(calls).toBe(2);
  });
});
