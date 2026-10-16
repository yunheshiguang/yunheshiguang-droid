export interface WorkspaceWindowOptions {
  label: string;
  url: string;
  title: string;
  width: number;
  height: number;
  decorations?: boolean;
  resizable?: boolean;
}

export interface WindowGateway {
  minimize(): Promise<void>;
  close(): Promise<void>;
  toggleMaximize(): Promise<void>;
  isMaximized(): Promise<boolean>;
  openWorkspaceWindow(options: WorkspaceWindowOptions): Promise<void>;
  openExternal(url: string): Promise<void>;
}
