import { describe, expect, it } from "vitest";
import { kmToMiles } from "../src/tasks/063-km-to-miles";

// Remove `.skip` once you start the task.
describe.skip("kmToMiles", () => {
  it("works", () => {
    expect(kmToMiles(10)).toEqual(6.21);
    expect(kmToMiles(0)).toEqual(0);
  });
});
