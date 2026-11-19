import { describe, expect, it } from "vitest";
import { swapCase } from "../src/tasks/086-swap-case";

// Remove `.skip` once you start the task.
describe.skip("swapCase", () => {
  it("works", () => {
    expect(swapCase('Hello')).toEqual('hELLO');
    expect(swapCase('a1B2')).toEqual('A1b2');
  });
});
