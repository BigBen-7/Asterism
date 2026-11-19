import { describe, expect, it } from "vitest";
import { pipe } from "../src/tasks/098-pipe";

// Remove `.skip` once you start the task.
describe.skip("pipe", () => {
  it("works", () => {
    const inc = (n: number) => n + 1;
    const dbl = (n: number) => n * 2;
    expect(pipe(inc, dbl)(5)).toBe(12);
    expect(pipe(dbl, inc)(5)).toBe(11);
    expect(pipe()(7)).toBe(7);
  });
});
