import { WORKSPACE_EVENTS } from "@/shared/api";
import { localApiProcedures, rpcProcedures } from "@/shared/proto";

export const procedures = localApiProcedures;

export const windowProcedures = rpcProcedures;

export const httpEndpoints = [
  { id: "health", method: "GET", path: "/v1/health" },
  { id: "events", method: "GET", path: "/v1/events" },
  { id: "openapi", method: "GET", path: "/v1/openapi.json" },
] as const;

export const pushedEvents = [
  { id: "devicesChanged", name: WORKSPACE_EVENTS.devicesChanged },
  { id: "documentsChanged", name: WORKSPACE_EVENTS.documentsChanged },
  { id: "deviceProfilesChanged", name: WORKSPACE_EVENTS.deviceProfilesChanged },
  { id: "automationChanged", name: WORKSPACE_EVENTS.automationChanged },
  { id: "cloudPhonesChanged", name: WORKSPACE_EVENTS.cloudPhonesChanged },
] as const;
