import { describe, expect, it } from "vitest";
import { formatWithCommas } from "../src/tasks/066-format-with-commas";

// Remove `.skip` once you start the task.
describe.skip("formatWithCommas", () => {
  it("works", () => {
    expect(formatWithCommas(1234567)).toEqual('1,234,567');
    expect(formatWithCommas(12)).toEqual('12');
  });
});
