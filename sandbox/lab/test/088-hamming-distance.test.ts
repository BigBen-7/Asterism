import { describe, expect, it } from "vitest";
import { hammingDistance } from "../src/tasks/088-hamming-distance";

// Remove `.skip` once you start the task.
describe.skip("hammingDistance", () => {
  it("works", () => {
    expect(hammingDistance('karolin', 'kathrin')).toEqual(3);
    expect(hammingDistance('abc', 'abc')).toEqual(0);
  });
});
