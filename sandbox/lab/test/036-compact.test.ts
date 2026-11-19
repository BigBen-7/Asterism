import { describe, expect, it } from "vitest";
import { compact } from "../src/tasks/036-compact";

// Remove `.skip` once you start the task.
describe.skip("compact", () => {
  it("works", () => {
    expect(compact([0, 1, '', 'a', null, false, 2])).toEqual([1, 'a', 2]);
  });
});
