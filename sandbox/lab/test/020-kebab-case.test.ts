import { describe, expect, it } from "vitest";
import { kebabCase } from "../src/tasks/020-kebab-case";

// Remove `.skip` once you start the task.
describe.skip("kebabCase", () => {
  it("works", () => {
    expect(kebabCase('Hello World')).toEqual('hello-world');
    expect(kebabCase('foo  bar')).toEqual('foo-bar');
  });
});
