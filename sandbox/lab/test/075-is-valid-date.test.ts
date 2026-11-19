import { describe, expect, it } from "vitest";
import { isValidDate } from "../src/tasks/075-is-valid-date";

// Remove `.skip` once you start the task.
describe.skip("isValidDate", () => {
  it("works", () => {
    expect(isValidDate('2024-02-29')).toEqual(true);
    expect(isValidDate('2023-02-29')).toEqual(false);
    expect(isValidDate('nope')).toEqual(false);
  });
});
