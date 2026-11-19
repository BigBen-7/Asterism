import { describe, expect, it } from "vitest";
import { countWords } from "../src/tasks/024-count-words";

// Remove `.skip` once you start the task.
describe.skip("countWords", () => {
  it("works", () => {
    expect(countWords('one two  three')).toEqual(3);
    expect(countWords('')).toEqual(0);
  });
});
