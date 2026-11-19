import { describe, expect, it } from "vitest";
import { bytesToHuman } from "../src/tasks/064-bytes-to-human";

// Remove `.skip` once you start the task.
describe.skip("bytesToHuman", () => {
  it("works", () => {
    expect(bytesToHuman(0)).toEqual('0 B');
    expect(bytesToHuman(1536)).toEqual('1.5 KB');
    expect(bytesToHuman(1048576)).toEqual('1 MB');
  });
});
