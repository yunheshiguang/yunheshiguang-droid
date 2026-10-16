export type PathPickRequest =
  | { target: "file"; extensions?: readonly string[] }
  | { target: "directory" }
  | { target: "save"; suggestedName: string; extensions?: readonly string[] };

export interface DialogGateway {
  pickPath(request: PathPickRequest): Promise<string | null>;
}
