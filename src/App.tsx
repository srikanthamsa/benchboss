import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PhysicalDevice, RoutingTable, Screen } from "./types";
import AssignScreen from "./components/AssignScreen";
import SlotGrid from "./components/SlotGrid";
import BenchPlayer from "./components/BenchPlayer";
import SwapModal from "./components/SwapModal";

const DEFAULT_PLAYER_NAMES = ["Srikant", "KVD", "Ashpak", "Ekansh", "Debu"];

export default function App() {
  const [screen, setScreen] = useState<Screen>("assign");
  const [devices, setDevices] = useState<PhysicalDevice[]>([]);
  const [routing, setRouting] = useState<RoutingTable>({
    slots: [null, null, null, null],
    bench: null,
  });
  const [swapModalOpen, setSwapModalOpen] = useState(false);
  const [forwarding, setForwarding] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Poll devices every 500ms while on assign screen
  useEffect(() => {
    if (screen !== "assign") return;

    const poll = async () => {
      try {
        const devs = await invoke<PhysicalDevice[]>("list_devices");
        setDevices(devs);
      } catch (e) {
        // On non-Windows or ViGEm not installed, show a friendly message
        setError(String(e));
      }
    };

    poll();
    const id = setInterval(poll, 500);
    return () => clearInterval(id);
  }, [screen]);

  // Poll routing table every 500ms while on main screen
  useEffect(() => {
    if (screen !== "main") return;

    const poll = async () => {
      try {
        const [devs, table] = await Promise.all([
          invoke<PhysicalDevice[]>("list_devices"),
          invoke<RoutingTable>("get_routing_table"),
        ]);
        setDevices(devs);
        setRouting(table);
      } catch (e) {
        setError(String(e));
      }
    };

    poll();
    const id = setInterval(poll, 500);
    return () => clearInterval(id);
  }, [screen]);

  const handleStartGame = useCallback(
    async (playerNames: string[]) => {
      try {
        // Assign names to detected devices in order
        for (let i = 0; i < Math.min(devices.length, playerNames.length); i++) {
          await invoke("assign_player", {
            handle: devices[i].handle,
            name: playerNames[i],
          });
        }

        // Assign first 4 devices to slots, 5th to bench
        for (let slot = 0; slot < Math.min(4, devices.length); slot++) {
          await invoke("assign_slot", { slot, handle: devices[slot].handle });
        }
        if (devices.length >= 5) {
          await invoke("assign_bench", { handle: devices[4].handle });
        }

        // Start the forwarding loop
        await invoke("start_forwarding");
        setForwarding(true);
        setScreen("main");
      } catch (e) {
        setError(String(e));
      }
    },
    [devices]
  );

  const handleBenchSlot = useCallback(
    async (slotIndex: number) => {
      const slotHandle = routing.slots[slotIndex];
      const benchHandle = routing.bench;
      if (slotHandle === null || benchHandle === null) return;
      try {
        await invoke("swap", { handleA: slotHandle, handleB: benchHandle });
        const table = await invoke<RoutingTable>("get_routing_table");
        setRouting(table);
      } catch (e) {
        setError(String(e));
      }
    },
    [routing]
  );

  const handleSwap = useCallback(
    async (handleA: number, handleB: number) => {
      try {
        await invoke("swap", { handleA, handleB });
        const table = await invoke<RoutingTable>("get_routing_table");
        setRouting(table);
        setSwapModalOpen(false);
      } catch (e) {
        setError(String(e));
      }
    },
    []
  );

  const handleStopForwarding = useCallback(async () => {
    try {
      await invoke("stop_forwarding");
      setForwarding(false);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const resolveDevice = (handle: number | null): PhysicalDevice | null => {
    if (handle === null) return null;
    return devices.find((d) => d.handle === handle) ?? null;
  };

  return (
    <div style={{ padding: "24px", minHeight: "100vh" }}>
      <header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: "24px",
        }}
      >
        <h1
          style={{
            fontSize: "28px",
            fontWeight: 700,
            color: "#7c3aed",
            letterSpacing: "2px",
            textTransform: "uppercase",
          }}
        >
          BenchBoss
        </h1>
        {screen === "main" && (
          <div style={{ display: "flex", gap: "12px", alignItems: "center" }}>
            <span
              style={{
                fontSize: "12px",
                color: forwarding ? "#22c55e" : "#ef4444",
                fontWeight: 600,
              }}
            >
              {forwarding ? "● LIVE" : "● STOPPED"}
            </span>
            <button
              onClick={
                forwarding ? handleStopForwarding : () => invoke("start_forwarding").then(() => setForwarding(true))
              }
              style={buttonStyle(forwarding ? "#7f1d1d" : "#14532d")}
            >
              {forwarding ? "Stop" : "Resume"}
            </button>
            <button
              onClick={() => {
                setScreen("assign");
                setForwarding(false);
              }}
              style={buttonStyle("#1f2937")}
            >
              ← Back
            </button>
          </div>
        )}
      </header>

      {error && (
        <div
          style={{
            background: "#7f1d1d",
            border: "1px solid #ef4444",
            borderRadius: "8px",
            padding: "12px 16px",
            marginBottom: "16px",
            fontSize: "13px",
            color: "#fca5a5",
          }}
        >
          <strong>Error:</strong> {error}
          <button
            onClick={() => setError(null)}
            style={{ marginLeft: "12px", background: "none", border: "none", color: "#f87171", cursor: "pointer" }}
          >
            ✕
          </button>
        </div>
      )}

      {screen === "assign" && (
        <AssignScreen
          devices={devices}
          defaultNames={DEFAULT_PLAYER_NAMES}
          onStart={handleStartGame}
        />
      )}

      {screen === "main" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
          <SlotGrid
            slots={routing.slots}
            resolveDevice={resolveDevice}
            onBenchSlot={handleBenchSlot}
          />
          <BenchPlayer
            device={resolveDevice(routing.bench)}
            onEnterGame={() => setSwapModalOpen(true)}
          />
        </div>
      )}

      {swapModalOpen && routing.bench !== null && (
        <SwapModal
          benchDevice={resolveDevice(routing.bench)!}
          slots={routing.slots}
          resolveDevice={resolveDevice}
          onSwap={handleSwap}
          onClose={() => setSwapModalOpen(false)}
        />
      )}
    </div>
  );
}

function buttonStyle(bg: string): React.CSSProperties {
  return {
    background: bg,
    color: "#e0e0e0",
    border: "1px solid #374151",
    borderRadius: "6px",
    padding: "6px 14px",
    fontSize: "13px",
    cursor: "pointer",
    fontFamily: "inherit",
  };
}
