import { describe, expect, it } from "vitest";
import { isAnagram } from "../src/tasks/081-is-anagram";

// Remove `.skip` once you start the task.
describe.skip("isAnagram", () => {
  it("works", () => {
    expect(isAnagram('listen', 'silent')).toEqual(true);
    expect(isAnagram('Dormitory', 'dirty room')).toEqual(true);
    expect(isAnagram('abc', 'abd')).toEqual(false);
  });
});
