import { createElement, useLayoutEffect, useRef, useState } from "react";

const SHADOW_STYLE_ID = "tamux-webview-stretch-style";

export function WebviewFrame({
    activeWorkspaceId,
    webviewRef,
    webBrowserUrl,
}: {
    activeWorkspaceId: string | null;
    webviewRef: React.RefObject<any>;
    webBrowserUrl: string;
}) {
    const frameRef = useRef<HTMLDivElement | null>(null);
    const [, setBounds] = useState({ width: 0, height: 0 });

    useLayoutEffect(() => {
        const frame = frameRef.current;
        if (!frame) return;

        let animationFrameId = 0;

        const applyBounds = () => {
            const width = Math.max(0, Math.floor(frame.clientWidth));
            const height = Math.max(0, Math.floor(frame.clientHeight));

            setBounds((current) => {
                if (current.width === width && current.height === height) {
                    return current;
                }
                return { width, height };
            });
        };

        const applyShadowStretch = () => {
            const webview = webviewRef.current;
            const shadowRoot = webview?.shadowRoot;
            if (!shadowRoot) {
                return;
            }

            let styleTag = shadowRoot.getElementById(SHADOW_STYLE_ID) as HTMLStyleElement | null;
            if (!styleTag) {
                styleTag = document.createElement("style");
                styleTag.id = SHADOW_STYLE_ID;
                styleTag.textContent = `
                    :host {
                        display: flex !important;
                        flex: 1 1 auto !important;
                        width: 100% !important;
                        height: 100% !important;
                        min-height: 0 !important;
                    }

                    iframe {
                        flex: 1 1 auto !important;
                        align-self: stretch !important;
                        width: 100% !important;
                        height: 100% !important;
                        min-height: 0 !important;
                        border: 0 !important;
                    }
                `;
                shadowRoot.appendChild(styleTag);
            }

            const iframe = shadowRoot.querySelector("iframe") as HTMLIFrameElement | null;
            if (iframe) {
                iframe.style.width = "100%";
                iframe.style.height = "100%";
                iframe.style.minHeight = "0";
                iframe.style.flex = "1 1 auto";
                iframe.style.alignSelf = "stretch";
            }
        };

        const scheduleShadowStretch = () => {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = window.requestAnimationFrame(() => {
                applyShadowStretch();
            });
        };

        applyBounds();
        scheduleShadowStretch();

        const webview = webviewRef.current;

        if (typeof webview.setAutoResize === "function") {
            try {
                webview.setAutoResize({ width: true, height: true });
            } catch {
                // noop
            }
        }

        const observer = new ResizeObserver(() => {
            applyBounds();
            scheduleShadowStretch();
        });

        observer.observe(frame);
        window.addEventListener("resize", applyBounds);
        window.addEventListener("resize", scheduleShadowStretch);

        return () => {
            observer.disconnect();
            window.removeEventListener("resize", applyBounds);
            window.removeEventListener("resize", scheduleShadowStretch);
            cancelAnimationFrame(animationFrameId);
        };
    }, [activeWorkspaceId, webviewRef, webBrowserUrl]);

    return (
        <div
            ref={frameRef}
            style={{
                flex: 1,
                minHeight: 0,
                height: "100%",
                minWidth: 0,
                display: "flex",
                overflow: "hidden",
                position: "relative",
            }}
        >
            {createElement("webview" as any, {
                key: activeWorkspaceId ?? "default-workspace",
                ref: webviewRef,
                src: webBrowserUrl,
                style: {
                    display: "block",
                    position: "absolute",
                    inset: 0,
                    width: "100%",
                    minWidth: 0,
                    minHeight: 0,
                    height: "100%",
                    border: "none",
                    background: "#ffffff",
                },
                allowpopups: "true",
                partition: "persist:amux-browser",
            })}
        </div>
    );
}
