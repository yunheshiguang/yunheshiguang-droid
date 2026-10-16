import { type ConnectRouter, createRouterTransport } from "@connectrpc/connect";

import { setRpcTransport } from "@/shared/api";

type Routes = (router: ConnectRouter) => void;

let installed: readonly Routes[] = [];

export function installFakeServices(...routes: Routes[]): void {
  const all = [...installed, ...routes];

  installed = all;
  setRpcTransport(
    createRouterTransport((router) => {
      for (const route of all) {
        route(router);
      }
    }),
  );
}

export function resetFakeServices(): void {
  installed = [];
}
