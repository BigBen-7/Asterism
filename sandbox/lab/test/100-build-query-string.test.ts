import { describe, expect, it } from "vitest";
import { buildQueryString } from "../src/tasks/100-build-query-string";

// Remove `.skip` once you start the task.
describe.skip("buildQueryString", () => {
  it("works", () => {
    expect(buildQueryString({ a: 1, b: 'two' })).toEqual('a=1&b=two');
    expect(buildQueryString({})).toEqual('');
  });
});
