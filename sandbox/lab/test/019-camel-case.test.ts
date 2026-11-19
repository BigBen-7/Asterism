import { describe, expect, it } from "vitest";
import { camelCase } from "../src/tasks/019-camel-case";

// Remove `.skip` once you start the task.
describe.skip("camelCase", () => {
  it("works", () => {
    expect(camelCase('hello world')).toEqual('helloWorld');
    expect(camelCase('Foo bar baz')).toEqual('fooBarBaz');
  });
});
