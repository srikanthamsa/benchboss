import type { PhysicalDevice } from "../types";

interface Props {
  device: PhysicalDevice | null;
  onEnterGame: () => void;
}

export default function BenchPlayer({ device, onEnterGame }: Props) {
  return (
    <div>
      <h2 style={{ fontSize: "16px", fontWeight: 600, color: "#a78bfa", marginBottom: "16px" }}>
        Bench
      </h2>
      <div
        style={{
          background: device ? "#1e3a5f" : "#111827",
          border: `2px solid ${device ? "#3b82f6" : "#374151"}`,
          borderRadius: "12px",
          padding: "20px 24px",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "16px",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
          {/* Bench badge */}
          <div
            style={{
              background: device ? "#1d4ed8" : "#374151",
              color: device ? "#bfdbfe" : "#9ca3af",
              borderRadius: "8px",
              padding: "8px 14px",
              fontSize: "13px",
              fontWeight: 700,
              letterSpacing: "1px",
            }}
          >
            BENCH
          </div>

          {/* Icon + name */}
          <div style={{ fontSize: "28px" }}>{device ? "🎮" : "🔲"}</div>
          <div>
            <div
              style={{
                fontWeight: 700,
                fontSize: "18px",
                color: device ? "#ffffff" : "#6b7280",
              }}
            >
              {device ? (device.player_name ?? `Handle 0x${device.handle.toString(16)}`) : "No player on bench"}
            </div>
            {device && (
              <div
                style={{
                  fontSize: "11px",
                  color: "#6b7280",
                  fontFamily: "monospace",
                  marginTop: "2px",
                }}
              >
                {device.device_path
                  ? device.device_path.split("\\").pop()
                  : `0x${device.handle.toString(16)}`}
              </div>
            )}
          </div>
        </div>

        {/* Enter game button */}
        {device && (
          <button
            onClick={onEnterGame}
            style={{
              background: "#1d4ed8",
              border: "1px solid #3b82f6",
              borderRadius: "8px",
              color: "#bfdbfe",
              padding: "10px 22px",
              fontSize: "14px",
              cursor: "pointer",
              fontFamily: "inherit",
              fontWeight: 600,
              whiteSpace: "nowrap",
            }}
          >
            Enter Game →
          </button>
        )}
      </div>
    </div>
  );
}
