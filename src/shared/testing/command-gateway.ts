import { type CommandGateway, type CommandPayload, setCommandGateway } from "@/shared/api";

type CommandHandler = (payload: CommandPayload | undefined) => unknown;

export type FakeCommandGateway = CommandGateway & {
  emit: (event: string, payload: unknown) => void;
};

export function createFakeCommandGateway(
  handlers: Readonly<Record<string, CommandHandler>>,
): FakeCommandGateway {
  const listeners = new Map<string, Set<(payload: unknown) => void>>();

  return {
    invoke: <TResult>(command: string, payload?: CommandPayload) => {
      const handler = handlers[command];

      return handler
        ? Promise.resolve(handler(payload)).then((value) => value as TResult)
        : new Promise<TResult>(() => {});
    },
    subscribe: async (event, handler) => {
      const subscribed = listeners.get(event) ?? new Set();

      subscribed.add(handler);
      listeners.set(event, subscribed);
      return () => subscribed.delete(handler);
    },
    emit: (event, payload) => {
      for (const handler of listeners.get(event) ?? []) {
        handler(payload);
      }
    },
  };
}

export function installFakeCommandGateway(
  handlers: Readonly<Record<string, CommandHandler>>,
): FakeCommandGateway {
  const gateway = createFakeCommandGateway(handlers);

  setCommandGateway(gateway);
  return gateway;
}
