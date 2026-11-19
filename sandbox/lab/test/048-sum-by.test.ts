import { describe, expect, it } from "vitest";
import { sumBy } from "../src/tasks/048-sum-by";

// Remove `.skip` once you start the task.
describe.skip("sumBy", () => {
  it("works", () => {
    expect(sumBy([{ n: 1 }, { n: 4 }], 'n')).toEqual(5);
    expect(sumBy([], 'n')).toEqual(0);
  });
});
