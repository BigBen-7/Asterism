import { describe, expect, it } from "vitest";
import { rgbToHex } from "../src/tasks/060-rgb-to-hex";

// Remove `.skip` once you start the task.
describe.skip("rgbToHex", () => {
  it("works", () => {
    expect(rgbToHex(255, 128, 0)).toEqual('#ff8000');
    expect(rgbToHex(0, 0, 0)).toEqual('#000000');
  });
});
