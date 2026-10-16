import type { ComponentType } from "react";

export type WorkspaceModuleId = string;

export type WorkspaceModuleStability = "stable" | "preview" | "experimental";

export type WorkspaceModuleIcon = ComponentType<{ className?: string }>;

export interface WorkspaceModuleNavigation {
  groupId: string;
  order?: number;
  labelKey: string;
  descriptionKey: string;
  icon: WorkspaceModuleIcon;
  hidden?: boolean;
}

export type WorkspaceModuleMessages = Readonly<Record<string, Readonly<Record<string, unknown>>>>;

export interface WorkspaceModuleDefinition {
  id: WorkspaceModuleId;
  component: ComponentType;
  navigation: WorkspaceModuleNavigation;
  messages?: WorkspaceModuleMessages;
  stability?: WorkspaceModuleStability;
  requires?: readonly string[];
  replaces?: readonly WorkspaceModuleId[];
}

export interface WorkspaceExtension {
  id: string;
  modules?: readonly WorkspaceModuleDefinition[];
  messages?: WorkspaceModuleMessages;
}

export interface WorkspaceNavigationGroupDefinition {
  id: string;
  order?: number;
  labelKey: string;
}

export interface WorkspaceNavigationEntry {
  id: WorkspaceModuleId;
  labelKey: string;
  descriptionKey: string;
  icon: WorkspaceModuleIcon;
  stability: WorkspaceModuleStability;
}

export interface WorkspaceNavigationGroup {
  id: string;
  labelKey: string;
  entries: readonly WorkspaceNavigationEntry[];
}

export interface WorkspaceRegistry {
  readonly modules: readonly WorkspaceModuleDefinition[];
  readonly navigation: readonly WorkspaceNavigationGroup[];
  readonly defaultModuleId: WorkspaceModuleId;
  readonly messages: WorkspaceModuleMessages;
  has(id: WorkspaceModuleId): boolean;
  resolve(id: WorkspaceModuleId): WorkspaceModuleDefinition | undefined;
  resolveOrDefault(id: WorkspaceModuleId): WorkspaceModuleDefinition | undefined;
}

export interface WorkspaceRegistryInput {
  modules: readonly unknown[];
  groups?: readonly WorkspaceNavigationGroupDefinition[];
  capabilities?: readonly string[];
  preferredDefaultModuleId?: WorkspaceModuleId;
}

export const DEFAULT_MODULE_ORDER = 1000;
