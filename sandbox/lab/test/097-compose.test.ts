import { describe, expect, it } from "vitest";
import { compose } from "../src/tasks/097-compose";

// Remove `.skip` once you start the task.
describe.skip("compose", () => {
  it("works", () => {
    const inc = (n: number) => n + 1;
    const dbl = (n: number) => n * 2;
    expect(compose(inc, dbl)(5)).toBe(11);
    expect(compose(dbl, inc)(5)).toBe(12);
    expect(compose()(7)).toBe(7);
  });
});
