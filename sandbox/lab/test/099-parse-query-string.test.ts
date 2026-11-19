import { describe, expect, it } from "vitest";
import { parseQueryString } from "../src/tasks/099-parse-query-string";

// Remove `.skip` once you start the task.
describe.skip("parseQueryString", () => {
  it("works", () => {
    expect(parseQueryString('a=1&b=two')).toEqual({ a: '1', b: 'two' });
    expect(parseQueryString('?x=y')).toEqual({ x: 'y' });
    expect(parseQueryString('')).toEqual({});
  });
});
