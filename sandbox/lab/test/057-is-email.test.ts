import { describe, expect, it } from "vitest";
import { isEmail } from "../src/tasks/057-is-email";

// Remove `.skip` once you start the task.
describe.skip("isEmail", () => {
  it("works", () => {
    expect(isEmail('a@b.co')).toEqual(true);
    expect(isEmail('nope')).toEqual(false);
    expect(isEmail('a@b')).toEqual(false);
    expect(isEmail('a b@c.de')).toEqual(false);
  });
});
