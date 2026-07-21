import { describe, expect, it } from "vitest";
import type { DeviceSnapshot } from "../contracts";
import { initialDeviceState, isNewer, reduceDeviceEvent } from "./deviceState";

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

  it("clears snapshot ordering when the sidecar session ends", () => {
    const connected = { snapshot: version(4, 20), error: null, connection: "connected" as const };
    expect(reduceDeviceEvent(connected, { event: "session_reset" })).toEqual(initialDeviceState);
  });
});
