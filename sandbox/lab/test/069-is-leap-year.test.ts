import { describe, expect, it } from "vitest";
import { isLeapYear } from "../src/tasks/069-is-leap-year";

// Remove `.skip` once you start the task.
describe.skip("isLeapYear", () => {
  it("works", () => {
    expect(isLeapYear(2000)).toEqual(true);
    expect(isLeapYear(1900)).toEqual(false);
    expect(isLeapYear(2024)).toEqual(true);
    expect(isLeapYear(2023)).toEqual(false);
  });
});
