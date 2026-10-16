import { createTauriCommandGateway } from "./tauri-command-gateway";
import { createTauriDialogGateway } from "./tauri-dialog-gateway";
import { createTauriLogGateway } from "./tauri-log-gateway";
import { createTauriRpcTransport } from "./tauri-rpc-transport";
import { createTauriWindowGateway } from "./tauri-window-gateway";

type GatewaySlot<T> = {
  get: () => T;
  set: (gateway: T | null) => void;
};

const overrideResets: (() => void)[] = [];

function gatewaySlot<T>(createDefault: () => T): GatewaySlot<T> {
  let override: T | null = null;
  let desktop: T | null = null;
  const slot: GatewaySlot<T> = {
    get: () => {
      if (override) {
        return override;
      }

      desktop ??= createDefault();

      return desktop;
    },
    set: (gateway) => {
      override = gateway;
    },
  };

  overrideResets.push(() => slot.set(null));

  return slot;
}

const commandSlot = gatewaySlot(createTauriCommandGateway);
const windowSlot = gatewaySlot(createTauriWindowGateway);
const logSlot = gatewaySlot(createTauriLogGateway);
const dialogSlot = gatewaySlot(createTauriDialogGateway);
const rpcSlot = gatewaySlot(createTauriRpcTransport);

export const commandGateway = commandSlot.get;
export const setCommandGateway = commandSlot.set;

export const windowGateway = windowSlot.get;
export const setWindowGateway = windowSlot.set;

export const logGateway = logSlot.get;
export const setLogGateway = logSlot.set;

export const dialogGateway = dialogSlot.get;
export const setDialogGateway = dialogSlot.set;

export const rpcTransport = rpcSlot.get;
export const setRpcTransport = rpcSlot.set;

export function resetGateways() {
  for (const reset of overrideResets) {
    reset();
  }
}
