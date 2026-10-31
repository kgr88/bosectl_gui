import { describe, expect, it } from "vitest";
import { isDeviceSnapshot } from "./contracts";

const snapshot = {
  identity: null,
  battery: null,
  firmware: null,
  connection: { state: "disconnected", reason: null, rfcomm_channel: null },
  capabilities: {
    noise_control: null,
    equalizer: [],
    settings: [],
    profiles: null,
  },
  values: { noise_control: null, eq: {}, settings: {}, profile_id: null },
  generation: 0,
  revision: 0,
};

describe("renderer boundary contracts", () => {
  it("accepts a capability-driven disconnected snapshot", () =>
    expect(isDeviceSnapshot(snapshot)).toBe(true));
  it("rejects a snapshot without monotonic version fields", () => {
    expect(isDeviceSnapshot({ ...snapshot, revision: -1 })).toBe(false);
  });
  it("rejects an impossible RFCOMM channel", () => {
    expect(
      isDeviceSnapshot({
        ...snapshot,
        connection: { ...snapshot.connection, rfcomm_channel: 31 },
      }),
    ).toBe(false);
  });
});
