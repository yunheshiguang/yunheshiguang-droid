import type {
  BatchActionKindJson,
  FlowTemplateCatalogJson,
  FlowTemplateJson,
} from "@/shared/proto";

const STARTER_SOURCE_ID = "core.starter";

function template(id: string, key: string, kinds: BatchActionKindJson[]): FlowTemplateJson {
  return {
    id,
    sourceId: STARTER_SOURCE_ID,
    name: { key: `workspace.templates.${key}.name` },
    description: { key: `workspace.templates.${key}.description` },
    steps: kinds.map((kind) => ({ kind, value: "" })),
  };
}

export function starterTemplateCatalog(): FlowTemplateCatalogJson {
  return {
    sources: [
      {
        id: STARTER_SOURCE_ID,
        label: { key: "workspace.templates.starter.label" },
        description: { key: "workspace.templates.starter.description" },
      },
    ],
    templates: [
      template("install-and-launch", "installAndLaunch", [
        "BATCH_ACTION_KIND_INSTALL_APK",
        "BATCH_ACTION_KIND_LAUNCH_APP",
      ]),
      template("capture-screens", "captureScreens", ["BATCH_ACTION_KIND_SCREENSHOT"]),
      template("recover-and-capture", "recoverAndCapture", [
        "BATCH_ACTION_KIND_RECOVER_CONNECTION",
        "BATCH_ACTION_KIND_SCREENSHOT",
      ]),
    ],
  };
}
