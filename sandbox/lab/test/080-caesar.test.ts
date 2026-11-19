import { describe, expect, it } from "vitest";
import { caesar } from "../src/tasks/080-caesar";

// Remove `.skip` once you start the task.
describe.skip("caesar", () => {
  it("works", () => {
    expect(caesar('abc', 1)).toEqual('bcd');
    expect(caesar('xyz', 3)).toEqual('abc');
    expect(caesar('Hello, World!', 13)).toEqual('Uryyb, Jbeyq!');
    expect(caesar('bcd', -1)).toEqual('abc');
  });
});
