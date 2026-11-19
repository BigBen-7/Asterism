import { describe, expect, it } from "vitest";
import { longestWord } from "../src/tasks/084-longest-word";

// Remove `.skip` once you start the task.
describe.skip("longestWord", () => {
  it("works", () => {
    expect(longestWord('a quick brownish fox')).toEqual('brownish');
    expect(longestWord('')).toEqual('');
  });
});
