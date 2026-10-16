import { assistantWorkspaceEn } from "./workspace-assistant";
import { scrcpyWorkspaceEn } from "./workspace-scrcpy";
import { auditWorkspaceEn, cloudWorkspaceEn, packagesWorkspaceEn } from "./workspace-sharing";

export const workspaceEn = {
  workspace: {
    assistant: assistantWorkspaceEn,
    audit: auditWorkspaceEn,
    packages: packagesWorkspaceEn,
    cloud: cloudWorkspaceEn,
    scrcpy: scrcpyWorkspaceEn,
    device: {
      group: {
        default: "Ungrouped",
      },
      note: {
        connected: "Connected",
        usbUnauthorized: "USB debugging not allowed yet",
        offline: "Offline. Reconnect the device",
        adbStatus: "ADB status: {{status}}",
      },
    },
    diagnostic: {
      title: {
        adbConflict: "ADB unavailable",
        deviceOffline: "Device offline",
        routerCongestion: "Unstable network",
        usbUnauthorized: "Waiting for device authorization",
        vendorPermission: "Vendor restriction",
        webview2Missing: "WebView2 missing",
        androidSecurityLimit: "Action blocked",
        androidPlatformToolsUnavailable: "Android platform-tools unavailable",
        androidPlatformToolsInvalidDirectory: "Invalid platform-tools folder",
        androidPlatformToolsAdbMissing: "adb not found",
        androidPlatformToolsFastbootMissing: "fastboot not found",
        androidPlatformToolsDownloadFailed: "Download failed",
        androidPlatformToolsArchiveInvalid: "Invalid package",
        androidPlatformToolsInstallFailed: "Install failed",
        androidPlatformToolsStateRecovered: "Settings reset",
        actionNotAuthorized: "Not authorized",
        none: "No issues",
        unknown: "Unknown status",
      },
      recovery: {
        adbConflict: "Make sure adb is installed and close any phone manager that is using ADB",
        deviceOffline:
          "Connect over USB and allow debugging, or reconnect over wireless debugging on the same network",
        routerCongestion: "Put fewer devices on the same router, or connect over USB or OTG",
        usbUnauthorized: "Allow USB debugging in the prompt on the device, then refresh",
        vendorPermission: "Check the USB install and security settings in Developer options",
        webview2Missing: "Install WebView2, then restart the app",
        androidSecurityLimit: "Check the required permissions and inputs, then try again",
        androidPlatformToolsUnavailable:
          "Install platform-tools, or choose a valid platform-tools folder",
        androidPlatformToolsInvalidDirectory:
          "Choose the extracted platform-tools folder, then apply again",
        androidPlatformToolsAdbMissing: "Make sure the selected folder contains adb",
        androidPlatformToolsFastbootMissing: "Make sure the selected folder contains fastboot",
        androidPlatformToolsDownloadFailed: "Check the network, then install again",
        androidPlatformToolsArchiveInvalid:
          "Download again, or choose a valid platform-tools folder",
        androidPlatformToolsInstallFailed:
          "Check folder permissions and disk space, then install again",
        androidPlatformToolsStateRecovered:
          "Review the platform-tools settings and apply them again if needed",
        actionNotAuthorized:
          "Ask an administrator to allow this action, or check the policy under Governance",
        none: "Nothing to do",
        unknown: "Check the device status again",
      },
      impact: {
        default: "The action did not finish",
        runtime: "{{message}}",
      },
    },
    connection: {
      sectionTitle: "Connection methods",
      usbFirst: {
        title: "First USB connection",
        step1: "Turn on Developer options",
        step2: "Turn on “USB debugging”",
        step3: "Allow USB debugging on the device",
        step4: "Refresh the device list",
      },
      wifi: {
        title: "Wireless debugging",
        step1:
          "In Developer options on the device, turn on “Wireless debugging” (Android 11 or later)",
        step2: "Make sure the computer and the device are on the same network",
        step3: "Tap “Pair device with pairing code”, then enter the code under nearby devices",
        step4: "Once paired, connect the device. Reconnect it if it drops",
        discoveryFailed: "Could not look for nearby devices: {{message}}",
      },
      cloud: {
        title: "Cloud phones",
        step1:
          "Register cloud phones by address under Cloud phones, or install an extension for your cloud phone service",
        step2: "Start the cloud phone. Once running, it appears under nearby devices",
        step3: "Click Connect. Once connected, it is marked as a cloud phone on the device wall",
      },
      otg: {
        title: "OTG and chassis",
        step1: "Check the cables and power",
        step2: "Make sure the authorization files are backed up",
        step3: "Make sure the host's ADB key is available",
        step4: "Then go to batch operations",
      },
      result: {
        connected: "Connected to {{device}}",
        alreadyConnected: "{{device}} is already connected",
        paired:
          "Paired with {{endpoint}}. Connect using the IP address and port shown under “Wireless debugging”",
        disconnected: "Disconnected {{device}}",
        invalidHost: "Enter an IP address or host name",
        invalidPort: "Enter a port between 1 and 65535",
        invalidPairingCode: "Enter the 6-digit pairing code shown on the device",
        invalidDevice: "Only network devices can be disconnected",
        unreachable: "Could not reach the device: {{reason}}",
        unauthorized:
          "The device refused the connection. Allow debugging on the device, then try again",
        pairingRejected: "Pairing failed. Check the pairing code, then try again",
        notConnected: "{{device}} is not connected",
        failed: "Connection failed: {{reason}}",
      },
    },
    governance: {
      provider: {
        localOperator: {
          label: "Local operator",
          description: "Anyone using this app can perform every action",
        },
        coreEntitlements: {
          label: "Built-in features",
          description: "No paid features are enabled; modules that need them are hidden",
        },
        sessionTrail: {
          label: "Recent actions",
          description: "Keeps the latest actions since the app started",
        },
        runLog: {
          label: "Run log",
          description: "Writes every action to the run log, which is kept across restarts",
        },
        localUser: {
          label: "Signed-in user",
          description:
            "Records the user signed in to this computer as the operator in the audit trail",
        },
        sqliteAudit: {
          label: "Local audit store",
          description:
            "Keeps audit entries on this computer, searchable and exportable after restarts",
        },
      },
      audit: {
        unfinished: "The action ended without reporting a result",
        interrupted: "The app quit before the action finished",
      },
    },
    documents: {
      storageFailed: "Could not change the document: {{message}}",
      audit: {
        saved: "Saved {{id}} in {{collection}}",
        removed: "Removed {{id}} from {{collection}}",
      },
    },
    telemetry: {
      health: {
        label: "Health summary",
        description: "Online rate, failure rate, and durations since the app started",
      },
    },
    vendors: {
      xiaomi: {
        label: "Xiaomi, Redmi, POCO",
        guidance: {
          securitySettings:
            "In Developer options, turn on “USB debugging (Security settings)”. This needs a SIM card and a signed-in Xiaomi account",
          installViaUsb: "Turn on “Install via USB” to install apps over USB",
        },
        explain: {
          securitySettings:
            "Xiaomi blocks simulated taps and text input until “USB debugging (Security settings)” is on",
          installViaUsb:
            "Xiaomi refused the install. Turn on “Install via USB” in Developer options",
        },
      },
      huawei: {
        label: "Huawei, Honor",
        guidance: {
          chargeOnly:
            "In Developer options, turn on “Allow ADB debugging in charge only mode”, or the device disconnects when USB is set to charging",
          alwaysAllow:
            "When the device asks to allow USB debugging, check “Always allow from this computer”",
        },
      },
      oppo: {
        label: "OPPO, OnePlus, realme",
        guidance: {
          permissionMonitoring:
            "In Developer options, turn on “Disable permission monitoring” so input and permissions can be controlled over USB",
          installPrompt:
            "The first time an app is installed over USB, confirm the install prompt on the device",
        },
        explain: {
          permissionMonitoring: "ColorOS blocks simulated input while permission monitoring is on",
          installPrompt:
            "The device did not confirm the install. Accept the prompt on the device, then try again",
        },
      },
      samsung: {
        label: "Samsung",
        guidance: {
          autoBlocker:
            "On One UI 6 or later, turn off “Auto Blocker” in Settings → Security and privacy, or it blocks commands over USB",
        },
      },
    },
    input: {
      yadb: {
        label: "YADB (built-in)",
        description:
          "No app to install and no keyboard to switch. Typing pastes through the clipboard and replaces its contents",
        pushFailed: "Could not push YADB to the device: {{message}}",
        failed: "YADB error: {{message}}",
        unconfirmed: "YADB could not write the clipboard, so no text was entered",
      },
      noChannel: "No installed input channel supports this device",
    },
    deviceShell: {
      failed: "Device command failed: {{message}}",
      invalid: "An argument contains a character the device command line does not support",
      tooLong: "The command is too long for the device command line",
    },
    organize: {
      invalid: {
        noDevices: "Select at least one device",
        aliasTooLong: "Name can be at most {{max}} characters",
        noteTooLong: "Note can be at most {{max}} characters",
        tagTooLong: "Each tag can be at most {{max}} characters",
        tooManyTags: "A device can have at most {{max}} tags",
        numberTooLarge: "Number can be at most {{max}}",
        groupMissing: "This group no longer exists",
        groupId: "Invalid group ID",
        groupNameRequired: "Enter a group name",
        groupNameTooLong: "Group name can be at most {{max}} characters",
      },
      storageFailed: "Could not save device profiles: {{message}}",
      audit: {
        profiles_one: "Updated the profile of {{count}} device",
        profiles_other: "Updated the profiles of {{count}} devices",
        groupSaved: "Saved group “{{name}}”",
        groupRemoved: "Deleted group “{{name}}”",
      },
    },
    scripts: {
      shell: {
        label: "Device shell (built-in)",
        description:
          "Runs scripts with the device's sh and returns the output and exit code, for up to 5 minutes",
      },
      uploadFailed: "Could not push the script to the device: {{message}}",
      noExitCode: "The script was cut off before it finished, so there is no exit code",
      noEngine: "No installed script engine can run this script",
      loadFailed: "Could not read the script library: {{message}}",
      missing: "Script not found in the script library",
      unreadable: "Could not parse this script",
      empty: "The script is empty",
    },
    automation: {
      loadFailed: "Could not read the flow library: {{message}}",
      flowMissing: "Flow not found in the flow library",
      flowUnreadable: "Could not parse this flow",
      flowEmpty: "The flow has no steps",
      targetMissing: "Select where to run",
      noDevices: "No device in the target is online",
      optionsTooLarge: "Run count, retries, or intervals exceed the limit",
      runNotRunning: "The run has already finished",
      stepFailed: "Step {{step}} did not finish on every device, so the run stopped",
      stopped: "Stopped on request",
      interrupted: "The app quit before the run finished",
      audit: {
        run: "Ran flow “{{name}}”",
        stopped: "Stopped flow “{{name}}”",
      },
    },
    schedules: {
      invalid: {
        id: "Invalid schedule ID",
        nameRequired: "Enter a schedule name",
        nameTooLong: "Schedule name can be at most {{max}} characters",
        flowRequired: "Select a flow to run",
        missing: "This schedule no longer exists",
        cron: "Invalid cron expression: {{message}}",
        limit: "You can have at most {{max}} schedules",
      },
      storageFailed: "Could not save the schedule: {{message}}",
      audit: {
        saved: "Saved schedule “{{name}}”",
        removed: "Deleted schedule “{{name}}”",
      },
    },
    api: {
      tokens: {
        label: "Local tokens (built-in)",
        description: "Issued in Settings; only digests are stored",
      },
      invalid: {
        tokenNameRequired: "Enter what the token is for",
        tokenNameTooLong: "The purpose can be at most {{max}} characters",
        tokenLimit: "You can have at most {{max}} tokens",
        tokenNameTaken: "Another token already has this purpose",
        tokenMissing: "This token no longer exists",
        port: "Enter a port between {{min}} and 65535",
      },
      storageFailed: "Could not save the local API settings: {{message}}",
      hostFiles: "The local API cannot use files on this computer. Run this action from the app",
      bindFailed: "Could not open the port: {{message}}",
      audit: {
        enabled: "Turned on the local API on port {{port}}",
        disabled: "Turned off the local API",
        tokenCreated: "Issued token “{{name}}”",
        tokenRevoked: "Revoked token “{{name}}”",
      },
      errors: {
        unauthorized: "Missing or invalid token",
        rateLimited: "Too many requests. Try again in {{seconds}} s",
        notFound: "Not found",
        badRequest: "Invalid request: {{message}}",
        unavailable: "Not available right now: {{message}}",
      },
    },
    updates: {
      unreachable: "Could not reach the release page: {{message}}",
      unreadable: "The release page sent information this version cannot read",
      storageFailed: "Could not save the update setting: {{message}}",
    },
    templates: {
      starter: {
        label: "Starter templates",
        description: "Common built-in flows",
      },
      installAndLaunch: {
        name: "Install and launch an app",
        description: "Install an APK on the selected devices, then launch it",
      },
      captureScreens: {
        name: "Screenshot each device",
        description: "Save screenshots of the selected devices to one folder",
      },
      recoverAndCapture: {
        name: "Reconnect, then screenshot",
        description: "Reconnect devices that dropped off, then take screenshots",
      },
    },
    task: {
      title: {
        screenshot: "Screenshot",
        screenRecord: "Screen record",
        installApk: "Install APK",
        uploadFile: "Upload file",
        inputText: "Input text",
        launchApp: "Launch app",
        clipboardWrite: "Write clipboard",
        recoverConnection: "Recover connection",
        runScript: "Run script",
        stopApp: "Stop app",
        clearAppData: "Clear app data",
        uninstallApp: "Uninstall app",
        default: "Batch operation",
      },
      target: {
        count_one: "{{count}} device",
        count_other: "{{count}} devices",
      },
      detail: {
        completed_one: "Completed on {{count}} device",
        completed_other: "Completed on {{count}} devices",
        saved_one: "Saved {{count}} file to {{path}}",
        saved_other: "Saved {{count}} files to {{path}}",
      },
    },
    validation: {
      needRealDevice: "Select at least one device",
      needText: "Enter the text to send",
      needPackage: "Enter a package name",
      needPath: "Choose a valid file",
      needSavePath: "Choose where to save",
      needScript: "Select a script to run",
      deviceNotConnected: "Device not connected",
    },
    adb: {
      unauthorized: "USB debugging is not allowed on {{serial}} yet",
      offline: "ADB reports the device as offline",
    },
    androidPlatformTools: {
      unavailable: "adb and fastboot are unavailable",
      invalidDirectory: "Could not read {{path}}",
      adbMissing: "{{binary}} not found",
      fastbootMissing: "{{binary}} not found",
    },
  },
} as const;
