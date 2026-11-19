import { describe, expect, it } from "vitest";
import { toBinary } from "../src/tasks/076-to-binary";

// Remove `.skip` once you start the task.
describe.skip("toBinary", () => {
  it("works", () => {
    expect(toBinary(5)).toEqual('101');
    expect(toBinary(0)).toEqual('0');
  });
});
