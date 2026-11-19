import { describe, expect, it } from "vitest";
import { diffDays } from "../src/tasks/074-diff-days";

// Remove `.skip` once you start the task.
describe.skip("diffDays", () => {
  it("works", () => {
    expect(diffDays('2024-01-01', '2024-01-31')).toEqual(30);
    expect(diffDays('2024-01-31', '2024-01-01')).toEqual(-30);
  });
});
