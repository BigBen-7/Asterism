import { describe, expect, it } from "vitest";
import { titleCase } from "../src/tasks/018-title-case";

// Remove `.skip` once you start the task.
describe.skip("titleCase", () => {
  it("works", () => {
    expect(titleCase('the quick brown fox')).toEqual('The Quick Brown Fox');
    expect(titleCase('hELLO wORLD')).toEqual('Hello World');
  });
});
