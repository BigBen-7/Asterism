import { describe, expect, it } from "vitest";
import { dayName } from "../src/tasks/071-day-name";

// Remove `.skip` once you start the task.
describe.skip("dayName", () => {
  it("works", () => {
    expect(dayName('2024-01-01')).toEqual('Monday');
    expect(dayName('2024-01-07')).toEqual('Sunday');
  });
});
