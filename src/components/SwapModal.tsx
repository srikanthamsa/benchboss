import type { PhysicalDevice } from "../types";

interface Props {
  benchDevice: PhysicalDevice;
  slots: (number | null)[];
  resolveDevice: (handle: number | null) => PhysicalDevice | null;
  onSwap: (handleA: number, handleB: number) => void;
  onClose: () => void;
}

const SLOT_ACCENT = ["#a78bfa", "#60a5fa", "#4ade80", "#f87171"];

export default function SwapModal({ benchDevice, slots, resolveDevice, onSwap, onClose }: Props) {
  const benchName = benchDevice.player_name ?? `0x${benchDevice.handle.toString(16)}`;

  return (
    // Backdrop
    <div
      onClick={onClose}
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.7)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
    >
      {/* Modal panel */}
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "#1f2937",
          border: "1px solid #4b5563",
          borderRadius: "16px",
          padding: "28px 32px",
          width: "420px",
          boxShadow: "0 24px 48px rgba(0,0,0,0.6)",
        }}
      >
        <h2 style={{ fontSize: "18px", fontWeight: 700, color: "#e0e0e0", marginBottom: "8px" }}>
          Sub {benchName} in
        </h2>
        <p style={{ fontSize: "13px", color: "#9ca3af", marginBottom: "24px" }}>
          Choose which slot player {benchName} will replace:
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: "10px", marginBottom: "24px" }}>
          {slots.map((handle, i) => {
            const device = resolveDevice(handle);
            const isEmpty = handle === null;

            return (
              <button
                key={i}
                disabled={isEmpty}
                onClick={() => {
                  if (handle !== null) {
                    onSwap(benchDevice.handle, handle);
                  }
                }}
                style={{
                  background: isEmpty ? "#111827" : "#111827",
                  border: `2px solid ${isEmpty ? "#374151" : SLOT_ACCENT[i]}`,
                  borderRadius: "10px",
                  padding: "14px 18px",
                  display: "flex",
                  alignItems: "center",
                  gap: "14px",
                  cursor: isEmpty ? "not-allowed" : "pointer",
                  textAlign: "left",
                  fontFamily: "inherit",
                  opacity: isEmpty ? 0.5 : 1,
                  transition: "background 0.15s",
                }}
                onMouseEnter={(e) => {
                  if (!isEmpty)
                    (e.currentTarget as HTMLButtonElement).style.background = "#374151";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLButtonElement).style.background = "#111827";
                }}
              >
                {/* Slot badge */}
                <span
                  style={{
                    background: isEmpty ? "#374151" : SLOT_ACCENT[i],
                    color: isEmpty ? "#9ca3af" : "#0f0f0f",
                    borderRadius: "4px",
                    padding: "2px 8px",
                    fontSize: "12px",
                    fontWeight: 700,
                    letterSpacing: "1px",
                    flexShrink: 0,
                  }}
                >
                  P{i + 1}
                </span>

                <span style={{ fontSize: "22px" }}>{isEmpty ? "🔲" : "🎮"}</span>

                <span
                  style={{
                    fontWeight: 600,
                    fontSize: "15px",
                    color: isEmpty ? "#6b7280" : "#e0e0e0",
                  }}
                >
                  {isEmpty
                    ? "Empty slot"
                    : (device?.player_name ?? `Handle 0x${handle?.toString(16)}`)}
                </span>

                {!isEmpty && (
                  <span
                    style={{
                      marginLeft: "auto",
                      fontSize: "12px",
                      color: "#6b7280",
                    }}
                  >
                    → bench
                  </span>
                )}
              </button>
            );
          })}
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button
            onClick={onClose}
            style={{
              background: "#374151",
              border: "1px solid #4b5563",
              borderRadius: "8px",
              color: "#9ca3af",
              padding: "8px 20px",
              fontSize: "14px",
              cursor: "pointer",
              fontFamily: "inherit",
            }}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
