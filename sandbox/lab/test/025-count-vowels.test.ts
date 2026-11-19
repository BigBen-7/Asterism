import { describe, expect, it } from "vitest";
import { countVowels } from "../src/tasks/025-count-vowels";

// Remove `.skip` once you start the task.
describe.skip("countVowels", () => {
  it("works", () => {
    expect(countVowels('Hello')).toEqual(2);
    expect(countVowels('xyz')).toEqual(0);
  });
});
