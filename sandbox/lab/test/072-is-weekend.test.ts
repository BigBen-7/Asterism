import { describe, expect, it } from "vitest";
import { isWeekend } from "../src/tasks/072-is-weekend";

// Remove `.skip` once you start the task.
describe.skip("isWeekend", () => {
  it("works", () => {
    expect(isWeekend('2024-01-06')).toEqual(true);
    expect(isWeekend('2024-01-08')).toEqual(false);
  });
});
