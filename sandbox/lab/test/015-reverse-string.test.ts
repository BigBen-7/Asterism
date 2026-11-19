import { describe, expect, it } from "vitest";
import { reverseString } from "../src/tasks/015-reverse-string";

// Remove `.skip` once you start the task.
describe.skip("reverseString", () => {
  it("works", () => {
    expect(reverseString('abc')).toEqual('cba');
    expect(reverseString('')).toEqual('');
  });
});
