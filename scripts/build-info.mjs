import { execSync } from "node:child_process";

const TIME_ZONE = "Asia/Singapore";

function runGit(command, fallback) {
  try {
    return execSync(`git ${command}`, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return fallback;
  }
}

function formatSingaporeTimestamp(date) {
  const parts = new Intl.DateTimeFormat("en-GB", {
    timeZone: TIME_ZONE,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    hourCycle: "h23",
  }).formatToParts(date);

  const values = Object.fromEntries(
    parts.filter((part) => part.type !== "literal").map((part) => [part.type, part.value]),
  );

  return `${values.year}${values.month}${values.day}-${values.hour}`;
}

export function getAppBuildInfo(now = new Date()) {
  const commitHistoryCount = Number.parseInt(runGit("rev-list --count HEAD", "0"), 10);
  const shortHash = runGit("rev-parse --short HEAD", "unknown");
  const versionCode = String(Number.isFinite(commitHistoryCount) ? commitHistoryCount + 1 : 1);

  return {
    versionCode,
    versionName: `${formatSingaporeTimestamp(now)}-${shortHash}`,
  };
}
