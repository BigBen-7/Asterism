import { describe, expect, it } from "vitest";
import { hexToRgb } from "../src/tasks/059-hex-to-rgb";

// Remove `.skip` once you start the task.
describe.skip("hexToRgb", () => {
  it("works", () => {
    expect(hexToRgb('#ff8000')).toEqual([255, 128, 0]);
    expect(hexToRgb('#000000')).toEqual([0, 0, 0]);
  });
});
