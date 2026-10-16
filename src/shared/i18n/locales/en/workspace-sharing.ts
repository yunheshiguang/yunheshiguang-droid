export const auditWorkspaceEn = {
  unavailable: "Audit history is unavailable. Only actions since the app started are shown",
  storageFailed: "Could not read or write the audit history: {{message}}",
  invalid: {
    format: "Select an export format",
    path: "Choose a writable location",
  },
  exported_one: "Exported {{count}} audit entry to {{path}}",
  exported_other: "Exported {{count}} audit entries to {{path}}",
  audit: {
    exported_one: "Exported {{count}} audit entry ({{format}})",
    exported_other: "Exported {{count}} audit entries ({{format}})",
  },
} as const;

export const packagesWorkspaceEn = {
  invalid: {
    id: "A package ID can contain only letters, digits, dots, hyphens, and underscores, and must start with a letter or digit",
    nameRequired: "Enter a package name",
    version: "Enter a semantic version, such as 1.2.0",
    textTooLong: "Text can be at most {{max}} characters",
    noFlows: "Select at least one flow",
    path: "Choose a valid location",
    origin: "Select a package to install",
    itemId: "{{id}} in the package is not a valid ID",
    duplicateId: "Duplicate ID in the package: {{id}}",
  },
  flowMissing: "Flow {{id}} is no longer in the flow library",
  scriptMissing: "Flow “{{flow}}” runs script {{script}}, which is no longer in the script library",
  marketMissing: "The template market no longer offers this package",
  readFailed: "Could not read the package file: {{message}}",
  tooLarge: "The package file can be at most {{max}} MB",
  unreadable: "Could not parse the package file: {{message}}",
  digestMismatch:
    "The package digest does not match. The file may be damaged or altered, so it cannot be installed",
  storageFailed: "Could not write the package: {{message}}",
  problem: {
    unverified: "Not signed by a signer you trust, so where it came from cannot be confirmed",
    scriptNotIncluded:
      "Flow “{{flow}}” runs script {{script}}, which is in neither the package nor the script library",
    scriptEmpty: "Script {{id}} is empty",
  },
  exported: "Exported to {{path}} (flows: {{flows}}, scripts: {{scripts}})",
  installed: "Installed the package (flows: {{flows}}, scripts: {{scripts}})",
  signatureInvalid: "The package signature is invalid, so it cannot be installed: {{message}}",
  signingFailed: "This computer's signing key is unavailable: {{message}}",
  signerInvalid: "Not a valid minisign public key",
  signerTrusted: "Packages signed by {{name}} will show as trusted",
  audit: {
    exported: "Exported template package “{{name}}” {{version}}",
    installed: "Installed template package “{{name}}” {{version}}",
    signerTrusted: "Trusted signer “{{name}}”",
  },
} as const;

export const cloudWorkspaceEn = {
  registered: {
    label: "Registered by address",
    description:
      "Cloud phones registered by adb address, in one list shared by the team. Start and stop are not supported",
  },
  unsupported: "This provider does not support this action",
  providerUnknown: "Unknown cloud phone provider",
  phoneMissing: "Cloud phone not found",
  invalid: {
    name: "Enter a cloud phone name of up to {{max}} characters",
    endpoint: "Enter a valid host and a port between 1 and 65535",
  },
  alreadyRegistered: "This address is already registered",
  timedOut: "The cloud phone service timed out",
  storageFailed: "Could not read or write the cloud phone list: {{message}}",
  audit: {
    started: "Started cloud phone “{{name}}”",
    stopped: "Stopped cloud phone “{{name}}”",
    registered: "Registered cloud phone “{{name}}”",
    removed: "Removed cloud phone “{{name}}”",
  },
} as const;
