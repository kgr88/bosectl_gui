import { describe, expect, it } from "vitest";
import type { DiscoveredDevice } from "../contracts";
import { durableConnectionTarget } from "./connectionTarget";

const device: DiscoveredDevice = {
  device_id: "AA:BB:CC:DD:EE:FF",
  address: "AA:BB:CC:DD:EE:FF",
  name: "Bose QuietComfort 35",
  product_id: 0x4020,
  device_type: "qc35",
  rfcomm_channel: 8,
  paired: true,
  supported: true,
};

describe("durableConnectionTarget", () => {
  it("converts a discovered device to a target that survives a sidecar restart", () => {
    expect(durableConnectionTarget({ kind: "discovered", device_id: device.device_id }, [device]))
      .toEqual({ kind: "manual", address: device.address, device_type: "qc35" });
  });

  it("leaves an unresolved discovery target unchanged", () => {
    const target = { kind: "discovered", device_id: "missing" } as const;
    expect(durableConnectionTarget(target, [])).toEqual(target);
  });
});
