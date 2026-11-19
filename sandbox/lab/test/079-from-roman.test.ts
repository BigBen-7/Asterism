import { describe, expect, it } from "vitest";
import { fromRoman } from "../src/tasks/079-from-roman";

// Remove `.skip` once you start the task.
describe.skip("fromRoman", () => {
  it("works", () => {
    expect(fromRoman('IX')).toEqual(9);
    expect(fromRoman('MCMXCIV')).toEqual(1994);
  });
});
