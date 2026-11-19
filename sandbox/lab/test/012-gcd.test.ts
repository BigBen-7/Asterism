import { describe, expect, it } from "vitest";
import { gcd } from "../src/tasks/012-gcd";

// Remove `.skip` once you start the task.
describe.skip("gcd", () => {
  it("works", () => {
    expect(gcd(12, 18)).toEqual(6);
    expect(gcd(7, 13)).toEqual(1);
    expect(gcd(0, 5)).toEqual(5);
  });
});
