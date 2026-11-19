import { describe, expect, it } from "vitest";
import { get } from "../src/tasks/056-get";

// Remove `.skip` once you start the task.
describe.skip("get", () => {
  it("works", () => {
    expect(get({ a: { b: 5 } }, 'a.b')).toEqual(5);
    expect(get({ a: 1 }, 'x.y', 'none')).toEqual('none');
    expect(get({ a: { b: 0 } }, 'a.b', 9)).toEqual(0);
  });
});
