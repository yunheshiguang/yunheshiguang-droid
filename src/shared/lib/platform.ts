export type DesktopPlatform = "macos" | "windows" | "linux" | "unknown";

export function detectDesktopPlatform(): DesktopPlatform {
  if (typeof navigator === "undefined") {
    return "unknown";
  }

  const agent = navigator.userAgent.toLowerCase();

  if (agent.includes("mac os x") || agent.includes("macintosh")) {
    return "macos";
  }

  if (agent.includes("windows")) {
    return "windows";
  }

  if (agent.includes("linux") || agent.includes("x11") || agent.includes("wayland")) {
    return "linux";
  }

  return "unknown";
}
