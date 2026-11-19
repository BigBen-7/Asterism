import { describe, expect, it } from "vitest";
import { once } from "../src/tasks/096-once";

// Remove `.skip` once you start the task.
describe.skip("once", () => {
  it("works", () => {
    let calls = 0;
    const f = once(() => {
      calls += 1;
      return 42;
    });
    expect(f()).toBe(42);
    expect(f()).toBe(42);
    expect(calls).toBe(1);
  });
});
