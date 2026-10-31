import type { ConnectionTarget, DiscoveredDevice } from "../contracts";

export function durableConnectionTarget(
  target: ConnectionTarget,
  devices: DiscoveredDevice[],
): ConnectionTarget {
  if (target.kind === "manual") return target;
  const device = devices.find((item) => item.device_id === target.device_id);
  if (
    device?.address &&
    (device.device_type === "qc_ultra2" || device.device_type === "qc35")
  ) {
    return {
      kind: "manual",
      address: device.address,
      device_type: device.device_type,
    };
  }
  return target;
}
