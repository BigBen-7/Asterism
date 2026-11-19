import { describe, expect, it } from "vitest";
import { padLeft } from "../src/tasks/026-pad-left";

// Remove `.skip` once you start the task.
describe.skip("padLeft", () => {
  it("works", () => {
    expect(padLeft('5', 3, '0')).toEqual('005');
    expect(padLeft('abc', 2, '0')).toEqual('abc');
  });
});
