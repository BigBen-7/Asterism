import { describe, expect, it } from "vitest";
import { mostFrequentChar } from "../src/tasks/083-most-frequent-char";

// Remove `.skip` once you start the task.
describe.skip("mostFrequentChar", () => {
  it("works", () => {
    expect(mostFrequentChar('aabbb')).toEqual('b');
    expect(mostFrequentChar('abab')).toEqual('a');
    expect(mostFrequentChar('')).toEqual(undefined);
  });
});
