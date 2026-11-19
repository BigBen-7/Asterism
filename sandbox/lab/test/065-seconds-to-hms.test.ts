import { describe, expect, it } from "vitest";
import { secondsToHms } from "../src/tasks/065-seconds-to-hms";

// Remove `.skip` once you start the task.
describe.skip("secondsToHms", () => {
  it("works", () => {
    expect(secondsToHms(3661)).toEqual('01:01:01');
    expect(secondsToHms(59)).toEqual('00:00:59');
  });
});
