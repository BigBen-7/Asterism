import { describe, expect, it } from "vitest";
import { reverseWords } from "../src/tasks/031-reverse-words";

// Remove `.skip` once you start the task.
describe.skip("reverseWords", () => {
  it("works", () => {
    expect(reverseWords('hello big world')).toEqual('world big hello');
  });
});
