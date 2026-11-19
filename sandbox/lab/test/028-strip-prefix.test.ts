import { describe, expect, it } from "vitest";
import { stripPrefix } from "../src/tasks/028-strip-prefix";

// Remove `.skip` once you start the task.
describe.skip("stripPrefix", () => {
  it("works", () => {
    expect(stripPrefix('foobar', 'foo')).toEqual('bar');
    expect(stripPrefix('bar', 'foo')).toEqual('bar');
  });
});
