import { useState } from "react";
import type { PhysicalDevice } from "../types";

interface Props {
  devices: PhysicalDevice[];
  defaultNames: string[];
  onStart: (playerNames: string[]) => void;
}

export default function AssignScreen({ devices, defaultNames, onStart }: Props) {
  const [names, setNames] = useState<string[]>(defaultNames.slice(0, 5));

  const updateName = (index: number, value: string) => {
    setNames((prev) => {
      const next = [...prev];
      next[index] = value;
      return next;
    });
  };

  const canStart = devices.length >= 2; // need at least 1 slot + 1 bench

  return (
    <div>
      <p style={{ color: "#9ca3af", marginBottom: "24px", fontSize: "14px" }}>
        Connect your Parsec controllers, then assign player names below. Detected controllers are matched in
        connection order.
      </p>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: "24px",
          marginBottom: "32px",
        }}
      >
        {/* Player name configuration */}
        <div>
          <h2 style={{ fontSize: "16px", fontWeight: 600, color: "#a78bfa", marginBottom: "16px" }}>
            Player Names
          </h2>
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            {names.map((name, i) => (
              <div key={i} style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <span
                  style={{
                    width: "28px",
                    height: "28px",
                    borderRadius: "50%",
                    background: i < 4 ? "#4c1d95" : "#1e3a5f",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "12px",
                    fontWeight: 700,
                    color: "#c4b5fd",
                    flexShrink: 0,
                  }}
                >
                  {i < 4 ? `P${i + 1}` : "B"}
                </span>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => updateName(i, e.target.value)}
                  placeholder={`Player ${i + 1}`}
                  style={inputStyle}
                />
              </div>
            ))}
          </div>

          <p style={{ fontSize: "11px", color: "#6b7280", marginTop: "12px" }}>
            P1–P4 → game slots &nbsp;|&nbsp; B → bench (5th controller)
          </p>
        </div>

        {/* Detected controllers */}
        <div>
          <h2 style={{ fontSize: "16px", fontWeight: 600, color: "#a78bfa", marginBottom: "16px" }}>
            Detected Controllers ({devices.length})
          </h2>
          {devices.length === 0 ? (
            <div
              style={{
                border: "1px dashed #374151",
                borderRadius: "8px",
                padding: "32px",
                textAlign: "center",
                color: "#6b7280",
                fontSize: "13px",
              }}
            >
              <div style={{ fontSize: "32px", marginBottom: "8px" }}>🎮</div>
              Waiting for controllers…
              <br />
              <span style={{ fontSize: "11px" }}>Connect via Parsec or USB</span>
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              {devices.map((dev, i) => (
                <div key={dev.handle} style={deviceCardStyle}>
                  <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <span
                      style={{
                        width: "24px",
                        height: "24px",
                        borderRadius: "50%",
                        background: i < 4 ? "#4c1d95" : "#1e3a5f",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        fontSize: "11px",
                        fontWeight: 700,
                        color: "#c4b5fd",
                        flexShrink: 0,
                      }}
                    >
                      {i < 4 ? `P${i + 1}` : "B"}
                    </span>
                    <div>
                      <div style={{ fontWeight: 600, fontSize: "14px", color: "#e0e0e0" }}>
                        {dev.player_name ?? names[i] ?? `Controller ${i + 1}`}
                      </div>
                      <div
                        style={{
                          fontSize: "10px",
                          color: "#6b7280",
                          fontFamily: "monospace",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                          maxWidth: "220px",
                        }}
                      >
                        {dev.device_path || `handle: 0x${dev.handle.toString(16)}`}
                      </div>
                    </div>
                  </div>
                  <div
                    style={{
                      width: "8px",
                      height: "8px",
                      borderRadius: "50%",
                      background: "#22c55e",
                    }}
                  />
                </div>
              ))}
              {devices.length < 5 && (
                <div
                  style={{
                    border: "1px dashed #374151",
                    borderRadius: "8px",
                    padding: "12px",
                    textAlign: "center",
                    color: "#6b7280",
                    fontSize: "12px",
                  }}
                >
                  {5 - devices.length} more controller{5 - devices.length !== 1 ? "s" : ""} needed for full bench
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        <button
          onClick={() => onStart(names)}
          disabled={!canStart}
          style={{
            background: canStart ? "#7c3aed" : "#374151",
            color: canStart ? "#fff" : "#6b7280",
            border: "none",
            borderRadius: "8px",
            padding: "12px 32px",
            fontSize: "15px",
            fontWeight: 600,
            cursor: canStart ? "pointer" : "not-allowed",
            fontFamily: "inherit",
            letterSpacing: "0.5px",
            transition: "background 0.2s",
          }}
        >
          Start Game
        </button>
        {!canStart && (
          <span style={{ color: "#6b7280", fontSize: "13px" }}>
            Connect at least 2 controllers to begin
          </span>
        )}
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: "#1f2937",
  border: "1px solid #374151",
  borderRadius: "6px",
  padding: "7px 12px",
  color: "#e0e0e0",
  fontSize: "14px",
  fontFamily: "inherit",
  width: "100%",
  outline: "none",
};

const deviceCardStyle: React.CSSProperties = {
  background: "#1f2937",
  border: "1px solid #374151",
  borderRadius: "8px",
  padding: "10px 14px",
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
};
