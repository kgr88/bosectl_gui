import { describe, expect, it } from "vitest";
import type { DeviceSnapshot } from "../contracts";
import { isNewer } from "./deviceState";

function version(generation: number, revision: number): DeviceSnapshot {
  return { generation, revision } as DeviceSnapshot;
}

describe("snapshot ordering", () => {
  it("ignores revisions from an older connection generation", () => {
    expect(isNewer(version(2, 1), version(1, 999))).toBe(false);
  });
  it("accepts a newer generation even when its revision restarts", () => {
    expect(isNewer(version(2, 999), version(3, 0))).toBe(true);
  });
});
