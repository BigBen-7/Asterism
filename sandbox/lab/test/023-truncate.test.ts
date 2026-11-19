import { describe, expect, it } from "vitest";
import { truncate } from "../src/tasks/023-truncate";

// Remove `.skip` once you start the task.
describe.skip("truncate", () => {
  it("works", () => {
    expect(truncate('hello world', 8)).toEqual('hello w…');
    expect(truncate('hi', 5)).toEqual('hi');
  });
});
