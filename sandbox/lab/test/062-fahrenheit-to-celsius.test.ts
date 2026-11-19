import { describe, expect, it } from "vitest";
import { fahrenheitToCelsius } from "../src/tasks/062-fahrenheit-to-celsius";

// Remove `.skip` once you start the task.
describe.skip("fahrenheitToCelsius", () => {
  it("works", () => {
    expect(fahrenheitToCelsius(32)).toEqual(0);
    expect(fahrenheitToCelsius(212)).toEqual(100);
  });
});
