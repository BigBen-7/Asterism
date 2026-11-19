import { describe, expect, it } from "vitest";
import { daysInMonth } from "../src/tasks/070-days-in-month";

// Remove `.skip` once you start the task.
describe.skip("daysInMonth", () => {
  it("works", () => {
    expect(daysInMonth(2024, 2)).toEqual(29);
    expect(daysInMonth(2023, 2)).toEqual(28);
    expect(daysInMonth(2023, 4)).toEqual(30);
    expect(daysInMonth(2023, 12)).toEqual(31);
  });
});
