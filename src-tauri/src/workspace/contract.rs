pub use yunheshiguang_contract::yunheshiguang::workspace::v1::{
    AndroidPlatformToolsBinary, AndroidPlatformToolsBinaryKind, AndroidPlatformToolsConfig,
    AndroidPlatformToolsInstall, AndroidPlatformToolsInstallStatus,
    AndroidPlatformToolsPackageKind, AndroidPlatformToolsSource, AndroidPlatformToolsState,
    BatchActionKind, BatchActionRequest, BatchActionResult, ConnectionHealth, ConnectionPlan,
    ConnectionType, DeviceGroup, DeviceHint, DeviceInputRoute, DeviceLabels, DeviceRecord,
    DeviceScriptOutput, DeviceStatus, DevicesChanged, DiagnosticCode, DiagnosticSignal, GroupTone,
    InstallAndroidPlatformToolsRequest, PiiClass, TaskItem, TaskStatus, UiText,
    UpdateAndroidPlatformToolsConfigRequest, WorkspaceSnapshot,
};

pub use yunheshiguang_contract::yunheshiguang::input::v1::InputChannels;

pub use yunheshiguang_contract::yunheshiguang::scripts::v1::{
    Script, ScriptEngines, ScriptLanguage,
};

pub use yunheshiguang_contract::yunheshiguang::automation::v1::{
    AutomationChanged, Flow, FlowEntry, FlowRun, FlowRunResult, FlowRuns, FlowStep, Flows,
    RemoveScheduleRequest, RunOptions, RunStatus, RunTarget, RunTrigger, SaveScheduleRequest,
    Schedule, ScheduleResult, ScheduleState, Schedules, StartFlowRunRequest, StepRun, StepStatus,
    run_target,
};

pub use yunheshiguang_contract::yunheshiguang::api::v1::{
    ApiEvent, ApiGatewayConfig, ApiGatewayResult, ApiGatewayState, ApiGatewayStatus, ApiToken,
    CreateApiTokenRequest, RevokeApiTokenRequest, UpdateApiGatewayRequest,
};

pub use yunheshiguang_contract::yunheshiguang::profiles::v1::{
    DeviceGroupDefinition, DeviceProfile, DeviceProfilePatch, DeviceProfilesChanged,
    OrganizeResult, RemoveDeviceGroupRequest, SaveDeviceGroupRequest, UpdateDeviceProfilesRequest,
};

pub use yunheshiguang_contract::yunheshiguang::connection::v1::{
    ConnectDeviceRequest, ConnectionOutcome, ConnectionResult, DeviceDiscovery,
    DisconnectDeviceRequest, DiscoveredEndpoint, DiscoveryFailure, NetworkEndpoint,
    PairDeviceRequest,
};

pub use yunheshiguang_contract::yunheshiguang::governance::v1::{
    AuditEntry, AuditExportFormat, AuditOutcome, AuditPage, AuditQuery, AuditReport, AuditTally,
    ExportAuditRequest, ExportAuditResult, GovernanceProvider, GovernanceState, GovernedAction,
    Operator, ReportAuditRequest,
};

pub use yunheshiguang_contract::yunheshiguang::assistant::v1::{
    AssistantResult, AssistantSettings, AssistantState, DraftFlowRequest, DraftFlowResult,
    DraftIssue, DraftScriptRequest, DraftScriptResult, FlowDraft, ScriptDraft, StructuredOutput,
    UpdateAssistantRequest,
};

pub use yunheshiguang_contract::yunheshiguang::packages::v1::{
    ConflictPolicy, ExportPackageRequest, ExportPackageResult, InspectPackageRequest,
    InspectPackageResult, InstallPackageRequest, InstallPackageResult, MarketPackage,
    MarketPackages, PackageFlow, PackageItem, PackageManifest, PackageOrigin, PackagePreview,
    PackageScript, PackageSignature, PackageSigner, TemplatePackage, TrustPackageSignerRequest,
    TrustPackageSignerResult, package_origin,
};

pub use yunheshiguang_contract::yunheshiguang::cloud::v1::{
    CloudPhone, CloudPhoneRequest, CloudPhoneResult, CloudPhoneStatus, CloudPhones,
    CloudPhonesChanged, CloudProviderInfo, ProviderProblem, RegisterCloudPhoneRequest,
};

pub use yunheshiguang_contract::yunheshiguang::adb::v1::{
    AdbDevice as AdbDeviceContract, AdbDeviceActionResult,
    AdbDeviceState as AdbDeviceStateContract, AdbErrorKind as AdbErrorKindContract,
    AdbRuntimeState, AdbServerState, AdbTransportKind,
};

pub use yunheshiguang_contract::yunheshiguang::scrcpy::v1::{
    ScrcpyBinary, ScrcpyBinaryKind, ScrcpyBinarySource, ScrcpyCaptureOrientation, ScrcpyConfig,
    ScrcpyConnectionHealth, ScrcpyDiagnosticCode, ScrcpyDiagnosticSignal, ScrcpyInstall,
    ScrcpyInstallStatus, ScrcpyPackageKind, ScrcpySession, ScrcpySessionStatus, ScrcpyState,
    ScrcpyUiText,
};

pub use yunheshiguang_contract::yunheshiguang::telemetry::v1::{
    OperationHealth, ProbeHealth, TelemetrySinkInfo, WorkspaceHealth,
};

pub use yunheshiguang_contract::yunheshiguang::vendors::v1::{
    DeviceVendor, DeviceVendors, VendorProfile,
};
