import { describe, expect, it } from "vitest";
import { padRight } from "../src/tasks/027-pad-right";

// Remove `.skip` once you start the task.
describe.skip("padRight", () => {
  it("works", () => {
    expect(padRight('5', 3, '0')).toEqual('500');
    expect(padRight('abc', 2, '0')).toEqual('abc');
  });
});
