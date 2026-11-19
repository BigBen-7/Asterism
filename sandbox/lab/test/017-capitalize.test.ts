import { describe, expect, it } from "vitest";
import { capitalize } from "../src/tasks/017-capitalize";

// Remove `.skip` once you start the task.
describe.skip("capitalize", () => {
  it("works", () => {
    expect(capitalize('hello')).toEqual('Hello');
    expect(capitalize('')).toEqual('');
  });
});
