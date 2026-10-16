import type { Transport } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { Channel, invoke } from "@tauri-apps/api/core";

type ResponseHead = { status: number; headers: [string, string][] };
type StreamSignal = { end?: true; error?: string };

async function ipcFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  const request = new Request(input, init);
  const { signal } = request;
  signal.throwIfAborted();
  const body = new Uint8Array(await request.arrayBuffer());
  const channel = new Channel<ArrayBuffer | StreamSignal>();
  let controller!: ReadableStreamDefaultController<Uint8Array>;
  let settled = false;
  channel.onmessage = (message) => {
    if (settled) {
      return;
    }
    if (message instanceof ArrayBuffer) {
      controller.enqueue(new Uint8Array(message));
      return;
    }
    settled = true;
    if (message.error !== undefined) {
      controller.error(new Error(message.error));
    } else {
      controller.close();
    }
  };

  const headers: Record<string, string> = Object.fromEntries(request.headers);
  headers["x-rpc-path"] = new URL(request.url).pathname;
  headers["x-rpc-content-type"] = request.headers.get("content-type") ?? "";
  headers["x-rpc-channel"] = channel.toJSON();
  const head = invoke<ResponseHead>("rpc", body, { headers });
  const stop = () => {
    if (!settled) {
      settled = true;
      head.then(() => invoke("rpc_cancel", { channel: channel.id })).catch(() => undefined);
    }
  };
  const stream = new ReadableStream<Uint8Array>({
    start: (started) => {
      controller = started;
    },
    cancel: stop,
  });
  const aborted = new Promise<never>((_, reject) => {
    signal.addEventListener(
      "abort",
      () => {
        if (!settled) {
          controller.error(signal.reason);
        }
        stop();
        reject(signal.reason);
      },
      { once: true },
    );
  });
  const replied = await Promise.race([head, aborted]);

  return new Response(stream, { status: replied.status, headers: replied.headers });
}

export function createTauriRpcTransport(): Transport {
  return createConnectTransport({
    baseUrl: "http://ipc.localhost",
    fetch: ipcFetch,
    useBinaryFormat: true,
  });
}
