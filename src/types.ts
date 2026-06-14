export interface PhysicalDevice {
  handle: number;
  device_path: string;
  player_name: string | null;
}

export interface RoutingTable {
  slots: (number | null)[]; // 4 elements, each is a device handle or null
  bench: number | null;
}

export type Screen = "assign" | "main";

export interface PlayerAssignment {
  name: string;
  handle: number | null;
}
