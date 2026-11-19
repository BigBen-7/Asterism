import { describe, expect, it } from "vitest";
import { stripSuffix } from "../src/tasks/029-strip-suffix";

// Remove `.skip` once you start the task.
describe.skip("stripSuffix", () => {
  it("works", () => {
    expect(stripSuffix('foobar', 'bar')).toEqual('foo');
    expect(stripSuffix('foo', 'bar')).toEqual('foo');
  });
});
