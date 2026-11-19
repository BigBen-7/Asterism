import { describe, expect, it } from "vitest";
import { addDays } from "../src/tasks/073-add-days";

// Remove `.skip` once you start the task.
describe.skip("addDays", () => {
  it("works", () => {
    expect(addDays('2024-02-28', 2)).toEqual('2024-03-01');
    expect(addDays('2024-01-01', -1)).toEqual('2023-12-31');
  });
});
