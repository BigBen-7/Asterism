import { describe, expect, it } from "vitest";
import { celsiusToFahrenheit } from "../src/tasks/061-celsius-to-fahrenheit";

// Remove `.skip` once you start the task.
describe.skip("celsiusToFahrenheit", () => {
  it("works", () => {
    expect(celsiusToFahrenheit(0)).toEqual(32);
    expect(celsiusToFahrenheit(100)).toEqual(212);
  });
});
