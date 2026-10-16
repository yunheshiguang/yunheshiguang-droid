export type CommandPayload = Record<string, unknown>;

export type Unsubscribe = () => void;

export interface CommandGateway {
  invoke<TResult>(command: string, payload?: CommandPayload): Promise<TResult>;
  subscribe(event: string, handler: (payload: unknown) => void): Promise<Unsubscribe>;
}
