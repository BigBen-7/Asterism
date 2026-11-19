import { describe, expect, it } from "vitest";
import { countOccurrences } from "../src/tasks/087-count-occurrences";

// Remove `.skip` once you start the task.
describe.skip("countOccurrences", () => {
  it("works", () => {
    expect(countOccurrences('banana', 'an')).toEqual(2);
    expect(countOccurrences('aaa', 'aa')).toEqual(1);
    expect(countOccurrences('abc', 'z')).toEqual(0);
  });
});
