export interface LogGateway {
  error(message: string): void;
  warn(message: string): void;
}
