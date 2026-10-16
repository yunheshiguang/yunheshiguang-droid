export const workspaceQueryKeys = {
  all: () => ["workspace"] as const,
  extension: (plugin: string, command: string, args: Readonly<Record<string, unknown>>) =>
    [...workspaceQueryKeys.all(), "extension", plugin, command, args] as const,
};

export const workspaceMutationKeys = {
  refreshSnapshot: () => [...workspaceQueryKeys.all(), "refresh-snapshot"] as const,
  batchAction: () => [...workspaceQueryKeys.all(), "batch-action"] as const,
  flowRun: () => [...workspaceQueryKeys.all(), "flow-run"] as const,
  schedules: () => [...workspaceQueryKeys.all(), "write-schedules"] as const,
  apiGateway: () => [...workspaceQueryKeys.all(), "write-api-gateway"] as const,
  platformTools: () => [...workspaceQueryKeys.all(), "platform-tools"] as const,
  updatePlatformTools: () => [...workspaceMutationKeys.platformTools(), "update"] as const,
  installPlatformTools: () => [...workspaceMutationKeys.platformTools(), "install"] as const,
  revealLogs: () => [...workspaceQueryKeys.all(), "reveal-logs"] as const,
  connection: () => [...workspaceQueryKeys.all(), "connection"] as const,
  connectDevice: () => [...workspaceMutationKeys.connection(), "connect"] as const,
  pairDevice: () => [...workspaceMutationKeys.connection(), "pair"] as const,
  disconnectDevice: () => [...workspaceMutationKeys.connection(), "disconnect"] as const,
  documents: (collection: string) =>
    [...workspaceQueryKeys.all(), "write-documents", collection] as const,
  organize: () => [...workspaceQueryKeys.all(), "organize"] as const,
  assistant: () => [...workspaceQueryKeys.all(), "write-assistant"] as const,
  assistantDraft: () => [...workspaceQueryKeys.all(), "assistant-draft"] as const,
  packages: () => [...workspaceQueryKeys.all(), "write-packages"] as const,
  cloudPhones: () => [...workspaceQueryKeys.all(), "write-cloud-phones"] as const,
  scrcpy: () => [...workspaceQueryKeys.all(), "write-scrcpy"] as const,
  installScrcpy: () => [...workspaceMutationKeys.scrcpy(), "install"] as const,
  exportAudit: () => [...workspaceQueryKeys.all(), "export-audit"] as const,
  checkForUpdates: () => [...workspaceQueryKeys.all(), "check-for-updates"] as const,
  updateSettings: () => [...workspaceQueryKeys.all(), "write-update-settings"] as const,
};
