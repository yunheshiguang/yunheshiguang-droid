import type { StatusTone } from "@/shared/lib/status-tokens";
import { ConnectionOutcome } from "@/shared/proto";

const TONES = {
  [ConnectionOutcome.UNSPECIFIED]: "critical",
  [ConnectionOutcome.CONNECTED]: "positive",
  [ConnectionOutcome.ALREADY_CONNECTED]: "positive",
  [ConnectionOutcome.PAIRED]: "positive",
  [ConnectionOutcome.DISCONNECTED]: "positive",
  [ConnectionOutcome.INVALID_REQUEST]: "caution",
  [ConnectionOutcome.UNREACHABLE]: "critical",
  [ConnectionOutcome.REJECTED]: "caution",
  [ConnectionOutcome.NOT_CONNECTED]: "caution",
  [ConnectionOutcome.TOOLS_UNAVAILABLE]: "critical",
  [ConnectionOutcome.DENIED]: "critical",
  [ConnectionOutcome.FAILED]: "critical",
} as const satisfies Record<ConnectionOutcome, StatusTone>;

export function connectionTone(outcome: ConnectionOutcome): StatusTone {
  return TONES[outcome] ?? "critical";
}
