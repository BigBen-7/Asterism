import { describe, expect, it } from "vitest";
import { percent } from "../src/tasks/067-percent";

// Remove `.skip` once you start the task.
describe.skip("percent", () => {
  it("works", () => {
    expect(percent(1, 4)).toEqual(25);
    expect(percent(1, 3)).toEqual(33.3);
    expect(percent(5, 0)).toEqual(0);
  });
});
