import { useState } from "react";

const layers = [
  {
    id: "ui",
    label: "UI LAYER",
    color: "#C8F060",
    textColor: "#0a0a0a",
    modules: [
      {
        name: "TUI Frontend",
        crate: "ratatui",
        desc: "Terminal UI: download list, progress bars, speed graphs, keybindings",
        files: ["src/ui/tui/mod.rs", "src/ui/tui/widgets/", "src/ui/tui/views/"],
      },
      {
        name: "CLI Interface",
        crate: "clap",
        desc: "Command-line interface for scripting, automation, and daemon control",
        files: ["src/cli/mod.rs", "src/cli/commands.rs", "src/cli/args.rs"],
      },
      {
        name: "Notification System",
        crate: "notify-rust",
        desc: "Desktop notifications on download complete, failure, or queue empty",
        files: ["src/ui/notify.rs"],
      },
    ],
  },
  {
    id: "orchestration",
    label: "ORCHESTRATION LAYER",
    color: "#60C8F0",
    textColor: "#0a0a0a",
    modules: [
      {
        name: "Job Queue Manager",
        crate: "tokio",
        desc: "Manages download job lifecycle: Created → Queued → Active → Paused → Done → Failed",
        files: ["src/queue/mod.rs", "src/queue/job.rs", "src/queue/state_machine.rs"],
      },
      {
        name: "Scheduler",
        crate: "tokio-cron-scheduler",
        desc: "Time-based scheduling, concurrency limits, bandwidth throttling policy",
        files: ["src/scheduler/mod.rs", "src/scheduler/policy.rs"],
      },
      {
        name: "Config Manager",
        crate: "toml + serde",
        desc: "Declarative TOML config, NixOS/Home Manager compatible, runtime reload",
        files: ["src/config/mod.rs", "src/config/schema.rs", "config.toml"],
      },
    ],
  },
  {
    id: "bridge",
    label: "ARIA2 BRIDGE LAYER",
    color: "#F060C8",
    textColor: "#fff",
    modules: [
      {
        name: "RPC Client",
        crate: "reqwest + serde_json",
        desc: "JSON-RPC 2.0 client: addUri, pause, resume, remove, tellStatus, getFiles",
        files: ["src/aria2/rpc.rs", "src/aria2/methods.rs", "src/aria2/types.rs"],
      },
      {
        name: "WebSocket Event Listener",
        crate: "tokio-tungstenite",
        desc: "Real-time event stream from aria2: onDownloadStart, onDownloadComplete, onError",
        files: ["src/aria2/ws.rs", "src/aria2/events.rs"],
      },
      {
        name: "aria2 Process Manager",
        crate: "tokio::process",
        desc: "Spawns and supervises aria2c daemon, manages lifecycle and restart policy",
        files: ["src/aria2/daemon.rs", "src/aria2/supervisor.rs"],
      },
      {
        name: "Session Serializer",
        crate: "serde + bincode",
        desc: "Persists aria2 session files; maps internal job IDs to aria2 GIDs",
        files: ["src/aria2/session.rs"],
      },
    ],
  },
  {
    id: "integration",
    label: "INTEGRATION LAYER",
    color: "#F0A060",
    textColor: "#0a0a0a",
    modules: [
      {
        name: "Browser Extension Bridge",
        crate: "tokio + serde_json",
        desc: "Native messaging host: receives URL + cookies + headers from browser extension",
        files: ["src/integration/native_host.rs", "src/integration/messaging.rs"],
      },
      {
        name: "Browser Extension",
        crate: "(TypeScript / WebExtension API)",
        desc: "Chrome/Firefox extension: intercepts downloads, sniffs media, sends to bridge",
        files: ["extension/manifest.json", "extension/background.ts", "extension/content.ts"],
      },
      {
        name: "Clipboard Monitor",
        crate: "arboard",
        desc: "Watches clipboard for URLs matching configured patterns; prompts to download",
        files: ["src/integration/clipboard.rs"],
      },
      {
        name: "Media Sniffer",
        crate: "regex + reqwest",
        desc: "Detects HLS/DASH manifests, direct media URLs from sniffed browser traffic",
        files: ["src/integration/sniffer.rs", "src/integration/patterns.rs"],
      },
    ],
  },
  {
    id: "postprocess",
    label: "POST-PROCESSING LAYER",
    color: "#A060F0",
    textColor: "#fff",
    modules: [
      {
        name: "File Organizer",
        crate: "std::fs + mime_guess",
        desc: "Categorizes completed downloads into Videos, Music, Documents, Archives folders",
        files: ["src/postprocess/organizer.rs", "src/postprocess/rules.rs"],
      },
      {
        name: "FFmpeg Bridge",
        crate: "tokio::process",
        desc: "Shells out to FFmpeg: merges HLS/DASH fragments, remuxes to MP4/MKV",
        files: ["src/postprocess/ffmpeg.rs"],
      },
      {
        name: "Archive Extractor",
        crate: "zip + tar",
        desc: "Auto-extracts ZIP, tar.gz archives after download completes",
        files: ["src/postprocess/extractor.rs"],
      },
      {
        name: "Hook System",
        crate: "tokio::process",
        desc: "User-defined shell hooks: on_complete, on_error, on_queue_empty scripts",
        files: ["src/postprocess/hooks.rs"],
      },
    ],
  },
  {
    id: "persistence",
    label: "PERSISTENCE LAYER",
    color: "#60F0A0",
    textColor: "#0a0a0a",
    modules: [
      {
        name: "Database",
        crate: "sqlx + SQLite",
        desc: "Stores all job metadata, history, categories, tags — async, type-safe queries",
        files: ["src/db/mod.rs", "src/db/migrations/", "src/db/queries.rs"],
      },
      {
        name: "State Store",
        crate: "serde + toml",
        desc: "Persists queue state, scheduler state, and resume data across restarts",
        files: ["src/db/state.rs"],
      },
    ],
  },
];

const projectStructure = `rustaria/
├── src/
│   ├── main.rs                  # Entry point, daemon bootstrap
│   ├── lib.rs
│   ├── aria2/
│   │   ├── mod.rs
│   │   ├── daemon.rs            # aria2c process management
│   │   ├── rpc.rs               # JSON-RPC 2.0 client
│   │   ├── ws.rs                # WebSocket event listener
│   │   ├── events.rs            # Event types & handlers
│   │   ├── session.rs           # GID ↔ job ID mapping
│   │   └── types.rs             # Shared types
│   ├── queue/
│   │   ├── mod.rs
│   │   ├── job.rs               # Job struct & state machine
│   │   └── state_machine.rs
│   ├── scheduler/
│   │   ├── mod.rs
│   │   └── policy.rs
│   ├── config/
│   │   ├── mod.rs
│   │   └── schema.rs
│   ├── integration/
│   │   ├── native_host.rs       # Browser native messaging
│   │   ├── messaging.rs
│   │   ├── clipboard.rs
│   │   └── sniffer.rs
│   ├── postprocess/
│   │   ├── organizer.rs
│   │   ├── ffmpeg.rs
│   │   ├── extractor.rs
│   │   └── hooks.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── queries.rs
│   │   └── migrations/
│   └── ui/
│       ├── tui/
│       │   ├── mod.rs
│       │   ├── views/
│       │   └── widgets/
│       └── notify.rs
├── extension/                   # Browser extension (TypeScript)
│   ├── manifest.json
│   ├── background.ts
│   └── content.ts
├── config.toml                  # Default config
├── Cargo.toml
├── flake.nix                    # NixOS flake
└── home-manager/
    └── rustaria.nix            # HM module`;

export default function ProjectMap() {
  const [active, setActive] = useState(null);
  const [view, setView] = useState("layers"); // layers | structure

  const activeLayer = layers.find((l) => l.id === active);

  return (
    <div style={{
      minHeight: "100vh",
      background: "#080808",
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      color: "#e0e0e0",
      padding: "0",
    }}>
      {/* Header */}
      <div style={{
        borderBottom: "1px solid #222",
        padding: "28px 40px 24px",
        display: "flex",
        alignItems: "flex-end",
        justifyContent: "space-between",
        flexWrap: "wrap",
        gap: "16px",
      }}>
        <div>
          <div style={{ fontSize: "11px", letterSpacing: "4px", color: "#555", marginBottom: "6px" }}>
            PRODUCTION ARCHITECTURE
          </div>
          <h1 style={{
            fontSize: "clamp(22px, 4vw, 36px)",
            fontWeight: 800,
            margin: 0,
            letterSpacing: "-1px",
            color: "#fff",
          }}>
            ferrum<span style={{ color: "#C8F060" }}>-dl</span>
          </h1>
          <div style={{ fontSize: "12px", color: "#555", marginTop: "4px" }}>
            Rust download manager · aria2 backend · NixOS native
          </div>
        </div>
        <div style={{ display: "flex", gap: "8px" }}>
          {["layers", "structure"].map((v) => (
            <button key={v} onClick={() => { setView(v); setActive(null); }} style={{
              background: view === v ? "#C8F060" : "transparent",
              color: view === v ? "#080808" : "#555",
              border: "1px solid",
              borderColor: view === v ? "#C8F060" : "#2a2a2a",
              borderRadius: "4px",
              padding: "7px 16px",
              fontSize: "11px",
              letterSpacing: "2px",
              cursor: "pointer",
              fontFamily: "inherit",
              textTransform: "uppercase",
            }}>
              {v}
            </button>
          ))}
        </div>
      </div>

      {view === "layers" && (
        <div style={{ padding: "32px 40px" }}>
          {/* Legend */}
          <div style={{ display: "flex", gap: "24px", marginBottom: "32px", flexWrap: "wrap" }}>
            {layers.map((l) => (
              <div key={l.id} style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "11px", color: "#666" }}>
                <div style={{ width: "10px", height: "10px", borderRadius: "2px", background: l.color }} />
                {l.label}
              </div>
            ))}
          </div>

          {/* Stack diagram */}
          <div style={{ display: "flex", flexDirection: "column", gap: "3px", marginBottom: "40px" }}>
            {layers.map((layer) => (
              <div
                key={layer.id}
                onClick={() => setActive(active === layer.id ? null : layer.id)}
                style={{
                  background: active === layer.id ? layer.color : "#111",
                  border: `1px solid ${active === layer.id ? layer.color : "#1e1e1e"}`,
                  borderLeft: `4px solid ${layer.color}`,
                  borderRadius: "4px",
                  padding: "14px 20px",
                  cursor: "pointer",
                  transition: "all 0.15s ease",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
                  <span style={{
                    fontSize: "11px",
                    letterSpacing: "3px",
                    fontWeight: 700,
                    color: active === layer.id ? layer.textColor : layer.color,
                  }}>
                    {layer.label}
                  </span>
                  <span style={{
                    fontSize: "11px",
                    color: active === layer.id ? (layer.textColor === "#fff" ? "rgba(255,255,255,0.6)" : "rgba(0,0,0,0.5)") : "#444",
                  }}>
                    {layer.modules.map(m => m.name).join(" · ")}
                  </span>
                </div>
                <span style={{
                  fontSize: "16px",
                  color: active === layer.id ? layer.textColor : layer.color,
                  opacity: 0.7,
                }}>
                  {active === layer.id ? "▲" : "▼"}
                </span>
              </div>
            ))}
          </div>

          {/* Expanded modules */}
          {activeLayer && (
            <div>
              <div style={{
                fontSize: "11px",
                letterSpacing: "3px",
                color: activeLayer.color,
                marginBottom: "16px",
                fontWeight: 700,
              }}>
                {activeLayer.label} — MODULES
              </div>
              <div style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fill, minmax(300px, 1fr))",
                gap: "12px",
              }}>
                {activeLayer.modules.map((mod) => (
                  <div key={mod.name} style={{
                    background: "#0e0e0e",
                    border: `1px solid #1e1e1e`,
                    borderTop: `2px solid ${activeLayer.color}`,
                    borderRadius: "4px",
                    padding: "20px",
                  }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "10px" }}>
                      <div style={{ fontSize: "14px", fontWeight: 700, color: "#fff" }}>{mod.name}</div>
                      <div style={{
                        fontSize: "10px",
                        padding: "3px 8px",
                        background: "#1a1a1a",
                        border: `1px solid ${activeLayer.color}33`,
                        borderRadius: "3px",
                        color: activeLayer.color,
                        letterSpacing: "1px",
                        whiteSpace: "nowrap",
                        marginLeft: "8px",
                      }}>
                        {mod.crate}
                      </div>
                    </div>
                    <div style={{ fontSize: "12px", color: "#777", lineHeight: "1.6", marginBottom: "14px" }}>
                      {mod.desc}
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                      {mod.files.map((f) => (
                        <div key={f} style={{
                          fontSize: "11px",
                          color: "#444",
                          fontFamily: "inherit",
                          padding: "3px 8px",
                          background: "#0a0a0a",
                          borderRadius: "3px",
                          borderLeft: `2px solid #2a2a2a`,
                        }}>
                          {f}
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {!activeLayer && (
            <div style={{
              border: "1px dashed #1e1e1e",
              borderRadius: "4px",
              padding: "32px",
              textAlign: "center",
              color: "#333",
              fontSize: "12px",
              letterSpacing: "2px",
            }}>
              CLICK A LAYER TO EXPAND MODULES
            </div>
          )}

          {/* Flow diagram */}
          <div style={{ marginTop: "48px" }}>
            <div style={{ fontSize: "11px", letterSpacing: "3px", color: "#444", marginBottom: "20px" }}>
              DATA FLOW
            </div>
            <div style={{
              display: "flex",
              alignItems: "center",
              gap: "0",
              overflowX: "auto",
              paddingBottom: "8px",
            }}>
              {[
                { label: "Browser\nExtension", color: "#F0A060" },
                { label: "Native\nMessaging Host", color: "#F0A060" },
                { label: "Job Queue\nManager", color: "#60C8F0" },
                { label: "Scheduler", color: "#60C8F0" },
                { label: "aria2 RPC\nClient", color: "#F060C8" },
                { label: "aria2c\nDaemon", color: "#F060C8" },
                { label: "Post-\nProcessor", color: "#A060F0" },
                { label: "SQLite\nDB", color: "#60F0A0" },
              ].map((node, i, arr) => (
                <div key={i} style={{ display: "flex", alignItems: "center", flexShrink: 0 }}>
                  <div style={{
                    background: "#0e0e0e",
                    border: `1px solid ${node.color}44`,
                    borderTop: `2px solid ${node.color}`,
                    borderRadius: "4px",
                    padding: "10px 14px",
                    fontSize: "10px",
                    color: node.color,
                    textAlign: "center",
                    whiteSpace: "pre",
                    lineHeight: "1.5",
                    minWidth: "80px",
                  }}>
                    {node.label}
                  </div>
                  {i < arr.length - 1 && (
                    <div style={{ color: "#2a2a2a", fontSize: "16px", padding: "0 4px", flexShrink: 0 }}>→</div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {view === "structure" && (
        <div style={{ padding: "32px 40px" }}>
          <div style={{ fontSize: "11px", letterSpacing: "3px", color: "#444", marginBottom: "20px" }}>
            PROJECT FILE STRUCTURE
          </div>
          <pre style={{
            background: "#0a0a0a",
            border: "1px solid #1a1a1a",
            borderLeft: "3px solid #C8F060",
            borderRadius: "4px",
            padding: "28px 32px",
            fontSize: "12px",
            lineHeight: "1.8",
            color: "#aaa",
            overflowX: "auto",
            margin: 0,
          }}>
            {projectStructure.split("\n").map((line, i) => {
              const isDir = line.includes("/") && !line.includes("#") && !line.trim().startsWith("#");
              const isComment = line.includes("#");
              const isHighlight = line.includes("main.rs") || line.includes("flake.nix") || line.includes("Cargo.toml");
              return (
                <span key={i} style={{
                  display: "block",
                  color: isHighlight ? "#C8F060" : isComment ? "#444" : isDir ? "#60C8F0" : "#888",
                }}>
                  {line}
                </span>
              );
            })}
          </pre>

          {/* Cargo.toml deps */}
          <div style={{ marginTop: "32px" }}>
            <div style={{ fontSize: "11px", letterSpacing: "3px", color: "#444", marginBottom: "20px" }}>
              CARGO.TOML — KEY DEPENDENCIES
            </div>
            <div style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
              gap: "8px",
            }}>
              {[
                { crate: "tokio", features: "full", purpose: "Async runtime" },
                { crate: "reqwest", features: "json, cookies", purpose: "HTTP client for RPC" },
                { crate: "tokio-tungstenite", features: "native-tls", purpose: "WebSocket for aria2 events" },
                { crate: "serde", features: "derive", purpose: "Serialization" },
                { crate: "serde_json", features: "—", purpose: "JSON-RPC payloads" },
                { crate: "sqlx", features: "sqlite, runtime-tokio", purpose: "Async database" },
                { crate: "ratatui", features: "—", purpose: "Terminal UI" },
                { crate: "clap", features: "derive", purpose: "CLI argument parsing" },
                { crate: "toml", features: "—", purpose: "Config file parsing" },
                { crate: "notify-rust", features: "—", purpose: "Desktop notifications" },
                { crate: "arboard", features: "—", purpose: "Clipboard monitoring" },
                { crate: "mime_guess", features: "—", purpose: "File type detection" },
                { crate: "tracing", features: "—", purpose: "Structured logging" },
                { crate: "anyhow", features: "—", purpose: "Error handling" },
              ].map((dep) => (
                <div key={dep.crate} style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  background: "#0a0a0a",
                  border: "1px solid #161616",
                  borderRadius: "3px",
                  padding: "10px 14px",
                }}>
                  <div>
                    <span style={{ fontSize: "13px", color: "#C8F060", fontWeight: 700 }}>{dep.crate}</span>
                    <span style={{ fontSize: "10px", color: "#333", marginLeft: "8px" }}>{dep.features}</span>
                  </div>
                  <span style={{ fontSize: "11px", color: "#555" }}>{dep.purpose}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div style={{
        borderTop: "1px solid #161616",
        padding: "20px 40px",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        flexWrap: "wrap",
        gap: "12px",
        marginTop: "40px",
      }}>
        <div style={{ fontSize: "11px", color: "#333", letterSpacing: "2px" }}>
          6 LAYERS · 20 MODULES · ARIA2 BACKEND
        </div>
        <div style={{ display: "flex", gap: "16px" }}>
          {["Rust 2024", "aria2 1.37+", "SQLite 3", "NixOS flake"].map((tag) => (
            <span key={tag} style={{
              fontSize: "10px",
              padding: "4px 10px",
              border: "1px solid #1e1e1e",
              borderRadius: "3px",
              color: "#444",
              letterSpacing: "1px",
            }}>
              {tag}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}