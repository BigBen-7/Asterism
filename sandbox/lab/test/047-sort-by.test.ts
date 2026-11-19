import { describe, expect, it } from "vitest";
import { sortBy } from "../src/tasks/047-sort-by";

// Remove `.skip` once you start the task.
describe.skip("sortBy", () => {
  it("works", () => {
    expect(sortBy([{ n: 2 }, { n: 1 }], 'n')).toEqual([{ n: 1 }, { n: 2 }]);
  });
});
