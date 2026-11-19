import { describe, expect, it } from "vitest";
import { isPrime } from "../src/tasks/011-is-prime";

// Remove `.skip` once you start the task.
describe.skip("isPrime", () => {
  it("works", () => {
    expect(isPrime(2)).toEqual(true);
    expect(isPrime(1)).toEqual(false);
    expect(isPrime(15)).toEqual(false);
    expect(isPrime(17)).toEqual(true);
  });
});
