import { describe, expect, it } from "vitest";
import { isPalindrome } from "../src/tasks/016-is-palindrome";

// Remove `.skip` once you start the task.
describe.skip("isPalindrome", () => {
  it("works", () => {
    expect(isPalindrome('A man, a plan, a canal: Panama')).toEqual(true);
    expect(isPalindrome('hello')).toEqual(false);
  });
});
