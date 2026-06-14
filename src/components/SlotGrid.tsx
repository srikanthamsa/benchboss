import type { PhysicalDevice } from "../types";

interface Props {
  slots: (number | null)[];
  resolveDevice: (handle: number | null) => PhysicalDevice | null;
  onBenchSlot: (slotIndex: number) => void;
}

const SLOT_COLORS = ["#4c1d95", "#1e3a5f", "#14532d", "#7f1d1d"];
const SLOT_ACCENT = ["#a78bfa", "#60a5fa", "#4ade80", "#f87171"];

export default function SlotGrid({ slots, resolveDevice, onBenchSlot }: Props) {
  return (
    <div>
      <h2 style={{ fontSize: "16px", fontWeight: 600, color: "#a78bfa", marginBottom: "16px" }}>
        Active Slots
      </h2>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(4, 1fr)",
          gap: "16px",
        }}
      >
        {slots.map((handle, i) => {
          const device = resolveDevice(handle);
          const isEmpty = handle === null;

          return (
            <div
              key={i}
              style={{
                background: isEmpty ? "#111827" : SLOT_COLORS[i],
                border: `2px solid ${isEmpty ? "#374151" : SLOT_ACCENT[i]}`,
                borderRadius: "12px",
                padding: "20px 16px",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: "12px",
                minHeight: "160px",
                position: "relative",
              }}
            >
              {/* Slot badge */}
              <div
                style={{
                  position: "absolute",
                  top: "10px",
                  left: "10px",
                  background: isEmpty ? "#374151" : SLOT_ACCENT[i],
                  color: isEmpty ? "#9ca3af" : "#0f0f0f",
                  borderRadius: "4px",
                  padding: "2px 7px",
                  fontSize: "11px",
                  fontWeight: 700,
                  letterSpacing: "1px",
                }}
              >
                P{i + 1}
              </div>

              {/* Controller icon */}
              <div style={{ fontSize: "36px", marginTop: "12px" }}>
                {isEmpty ? "🔲" : "🎮"}
              </div>

              {/* Player name */}
              <div
                style={{
                  fontWeight: 700,
                  fontSize: "16px",
                  color: isEmpty ? "#6b7280" : "#ffffff",
                  textAlign: "center",
                }}
              >
                {isEmpty
                  ? "Empty"
                  : (device?.player_name ?? `Handle ${handle?.toString(16)}`)}
              </div>

              {/* Device path snippet */}
              {device && (
                <div
                  style={{
                    fontSize: "10px",
                    color: "#9ca3af",
                    fontFamily: "monospace",
                    textAlign: "center",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                    width: "100%",
                  }}
                >
                  {device.device_path
                    ? device.device_path.split("\\").pop()
                    : `0x${device.handle.toString(16)}`}
                </div>
              )}

              {/* Bench button */}
              {!isEmpty && (
                <button
                  onClick={() => onBenchSlot(i)}
                  style={{
                    background: "rgba(0,0,0,0.3)",
                    border: `1px solid ${SLOT_ACCENT[i]}`,
                    borderRadius: "6px",
                    color: SLOT_ACCENT[i],
                    padding: "5px 14px",
                    fontSize: "12px",
                    cursor: "pointer",
                    fontFamily: "inherit",
                    fontWeight: 600,
                    marginTop: "auto",
                  }}
                >
                  Bench
                </button>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
