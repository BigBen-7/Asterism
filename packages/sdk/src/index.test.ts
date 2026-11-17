import { expect, test } from "vitest";
import { SDK_VERSION } from "./index";

test("exports a version", () => {
  expect(SDK_VERSION).toBe("0.0.0");
});
