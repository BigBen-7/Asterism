import { describe, expect, it } from "vitest";
import { chunk } from "../src/tasks/033-chunk";

// Remove `.skip` once you start the task.
describe.skip("chunk", () => {
  it("works", () => {
    expect(chunk([1, 2, 3, 4, 5], 2)).toEqual([[1, 2], [3, 4], [5]]);
    expect(chunk([], 3)).toEqual([]);
  });
});
