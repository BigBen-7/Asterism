import { describe, expect, it } from "vitest";
import { slugify } from "../src/tasks/022-slugify";

// Remove `.skip` once you start the task.
describe.skip("slugify", () => {
  it("works", () => {
    expect(slugify('Hello, World!')).toEqual('hello-world');
    expect(slugify('  Many   spaces  ')).toEqual('many-spaces');
  });
});
