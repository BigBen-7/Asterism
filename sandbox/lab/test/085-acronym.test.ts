import { describe, expect, it } from "vitest";
import { acronym } from "../src/tasks/085-acronym";

// Remove `.skip` once you start the task.
describe.skip("acronym", () => {
  it("works", () => {
    expect(acronym('portable network graphics')).toEqual('PNG');
    expect(acronym('  as soon as possible ')).toEqual('ASAP');
  });
});
