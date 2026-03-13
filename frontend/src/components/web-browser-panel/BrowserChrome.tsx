import { navBtnStyle } from "./shared";

export function BrowserChrome({
    address,
    setAddress,
    pageTitle,
    back,
    forward,
    reload,
    navigate,
    toggleFullscreen,
    close,
}: {
    address: string;
    setAddress: (value: string) => void;
    pageTitle: string;
    back: () => void;
    forward: () => void;
    reload: () => void;
    navigate: (url: string) => void;
    toggleFullscreen: () => void;
    close: () => void;
}) {
    return (
        <>
            <div
                style={{
                    height: 40,
                    display: "flex",
                    alignItems: "center",
                    flexShrink: 0,
                    gap: 6,
                    padding: "0 10px",
                    borderBottom: "1px solid var(--border)",
                    background: "var(--bg-secondary)",
                }}
            >
                <button onClick={back} style={navBtnStyle} title="Back">←</button>
                <button onClick={forward} style={navBtnStyle} title="Forward">→</button>
                <button onClick={reload} style={navBtnStyle} title="Reload">↻</button>
                <input
                    value={address}
                    onChange={(event) => setAddress(event.target.value)}
                    onKeyDown={(event) => {
                        if (event.key === "Enter") {
                            navigate(address);
                        }
                    }}
                    style={{
                        flex: 1,
                        height: 26,
                        border: "1px solid var(--border)",
                        borderRadius: "var(--radius-sm)",
                        background: "var(--bg-primary)",
                        color: "var(--text-primary)",
                        padding: "0 8px",
                        fontFamily: "var(--font-mono)",
                        fontSize: 12,
                    }}
                />
                <button onClick={() => navigate(address)} style={navBtnStyle} title="Go">Go</button>
                <button onClick={toggleFullscreen} style={navBtnStyle} title="Toggle fullscreen">⛶</button>
                <button onClick={close} style={navBtnStyle} title="Close browser">✕</button>
            </div>

            <div
                style={{
                    height: 24,
                    display: "flex",
                    alignItems: "center",
                    flexShrink: 0,
                    padding: "0 10px",
                    borderBottom: "1px solid var(--border)",
                    color: "var(--text-muted)",
                    fontSize: 11,
                }}
            >
                {pageTitle || "Browser"}
            </div>
        </>
    );
}
