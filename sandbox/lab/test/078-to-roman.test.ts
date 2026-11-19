import { describe, expect, it } from "vitest";
import { toRoman } from "../src/tasks/078-to-roman";

// Remove `.skip` once you start the task.
describe.skip("toRoman", () => {
  it("works", () => {
    expect(toRoman(4)).toEqual('IV');
    expect(toRoman(1994)).toEqual('MCMXCIV');
    expect(toRoman(3)).toEqual('III');
  });
});
