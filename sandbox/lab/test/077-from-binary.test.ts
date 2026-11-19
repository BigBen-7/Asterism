import { describe, expect, it } from "vitest";
import { fromBinary } from "../src/tasks/077-from-binary";

// Remove `.skip` once you start the task.
describe.skip("fromBinary", () => {
  it("works", () => {
    expect(fromBinary('101')).toEqual(5);
    expect(fromBinary('0')).toEqual(0);
  });
});
