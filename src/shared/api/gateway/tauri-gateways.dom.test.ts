import { create, fromBinary, type MessageInitShape, toBinary } from "@bufbuild/protobuf";
import { createClient } from "@connectrpc/connect";
import { emit } from "@tauri-apps/api/event";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { errorMessage } from "@/shared/lib/error-message";
import {
  AssistantService,
  DraftScriptEventSchema,
  GetGovernanceStateRequestSchema,
  GovernanceService,
  GovernanceStateSchema,
  WorkspaceService,
} from "@/shared/proto";

import { createTauriCommandGateway } from "./tauri-command-gateway";
import { createTauriDialogGateway } from "./tauri-dialog-gateway";
import { createTauriLogGateway } from "./tauri-log-gateway";
import { createTauriRpcTransport } from "./tauri-rpc-transport";
import { createTauriWindowGateway } from "./tauri-window-gateway";

type Call = { cmd: string; payload: unknown };

let calls: Call[] = [];

function answer(respond: (cmd: string) => unknown = () => null, options?: { events: boolean }) {
  mockIPC(
    (cmd, payload) => {
      calls.push({ cmd, payload });
      return respond(cmd);
    },
    { shouldMockEvents: options?.events ?? false },
  );
}

const payloadsOf = (cmd: string) =>
  calls.filter((call) => call.cmd === cmd).map((call) => call.payload);

type Internals = {
  invoke: (
    cmd: string,
    args: unknown,
    options?: { headers?: Record<string, string> },
  ) => Promise<unknown>;
  runCallback: (id: number, data: unknown) => void;
};

function envelope(flags: number, data: Uint8Array): Uint8Array {
  const framed = new Uint8Array(5 + data.length);
  framed[0] = flags;
  new DataView(framed.buffer).setUint32(1, data.length);
  framed.set(data, 5);
  return framed;
}

type Reply = { status?: number; contentType: string; chunks: Uint8Array[]; open?: boolean };

function bridge(respond: (path: string) => Reply) {
  mockIPC(() => null);
  const internals = (window as unknown as { __TAURI_INTERNALS__: Internals }).__TAURI_INTERNALS__;
  const requests: { path: string; contentType: string; channel: number; body: Uint8Array }[] = [];
  const cancelled: unknown[] = [];

  internals.invoke = async (cmd, args, options) => {
    if (cmd === "rpc_cancel") {
      cancelled.push((args as { channel: number }).channel);
      return null;
    }
    if (cmd !== "rpc") {
      return null;
    }
    const headers = options?.headers ?? {};
    const path = headers["x-rpc-path"] ?? "";
    const channel = Number((headers["x-rpc-channel"] ?? "").replace("__CHANNEL__:", ""));
    requests.push({
      path,
      contentType: headers["x-rpc-content-type"] ?? "",
      channel,
      body: args as Uint8Array,
    });
    const { status = 200, contentType, chunks, open = false } = respond(path);
    queueMicrotask(() => {
      chunks.forEach((chunk, index) => {
        internals.runCallback(channel, { message: chunk.slice().buffer, index });
      });
      if (!open) {
        internals.runCallback(channel, { message: { end: true }, index: chunks.length });
      }
    });

    return { status, headers: [["content-type", contentType]] };
  };

  return Object.assign(requests, { cancelled });
}

const draftEvent = (init: MessageInitShape<typeof DraftScriptEventSchema>) =>
  envelope(0, toBinary(DraftScriptEventSchema, create(DraftScriptEventSchema, init)));

beforeEach(() => {
  calls = [];
  mockWindows("main");
});

afterEach(() => {
  clearMocks();
});

describe("Tauri RPC transport", () => {
  it("sends a Connect call over the IPC bridge and decodes the reply", async () => {
    const requests = bridge(() => ({
      contentType: "application/proto",
      chunks: [
        toBinary(GovernanceStateSchema, create(GovernanceStateSchema, { auditCapacity: 200 })),
      ],
    }));
    const governance = createClient(GovernanceService, createTauriRpcTransport());

    await expect(governance.getGovernanceState({})).resolves.toMatchObject({ auditCapacity: 200 });
    expect(requests[0]?.path).toBe(
      "/yunheshiguang.governance.v1.GovernanceService/GetGovernanceState",
    );
    expect(requests[0]?.contentType).toBe("application/proto");
    expect(
      fromBinary(GetGovernanceStateRequestSchema, requests[0]?.body ?? new Uint8Array()),
    ).toEqual(create(GetGovernanceStateRequestSchema));
  });

  it("delivers what a streaming call writes as it arrives", async () => {
    const requests = bridge(() => ({
      contentType: "application/connect+proto",
      chunks: [
        draftEvent({ event: { case: "text", value: "echo " } }),
        draftEvent({ event: { case: "text", value: "hi" } }),
        draftEvent({ event: { case: "result", value: { accepted: true } } }),
        envelope(2, new TextEncoder().encode("{}")),
      ],
    }));
    const received: string[] = [];

    for await (const { event: next } of createClient(
      AssistantService,
      createTauriRpcTransport(),
    ).draftScript({ prompt: "x" })) {
      received.push(next.case === "text" ? next.value : String(next.case));
    }

    expect(received).toEqual(["echo ", "hi", "result"]);
    expect(requests[0]?.contentType).toBe("application/connect+proto");
    expect(requests.cancelled).toEqual([]);
  });

  it("asks the backend to stop a stream the caller abandons", async () => {
    const requests = bridge(() => ({
      contentType: "application/connect+proto",
      chunks: [draftEvent({ event: { case: "text", value: "echo " } })],
      open: true,
    }));
    const abort = new AbortController();
    const received: string[] = [];

    const drafting = (async () => {
      for await (const { event: next } of createClient(
        AssistantService,
        createTauriRpcTransport(),
      ).draftScript({ prompt: "x" }, { signal: abort.signal })) {
        received.push(next.case === "text" ? next.value : String(next.case));
        abort.abort();
      }
    })();

    await expect(drafting).rejects.toThrow();
    expect(received).toEqual(["echo "]);
    await vi.waitFor(() => {
      expect(requests.cancelled).toEqual([requests[0]?.channel]);
    });
  });

  it("rejects with the message the backend gave when a call fails", async () => {
    bridge(() => ({
      status: 503,
      contentType: "application/json",
      chunks: [new TextEncoder().encode('{"code":"unavailable","message":"adb is not running"}')],
    }));

    const failure = await createClient(WorkspaceService, createTauriRpcTransport())
      .getWorkspaceSnapshot({})
      .catch((error: unknown) => error);

    expect(errorMessage(failure)).toBe("adb is not running");
  });
});

describe("Tauri command gateway", () => {
  it("passes host commands straight to the host", async () => {
    answer();

    await createTauriCommandGateway().invoke("reveal_log_directory");

    expect(payloadsOf("reveal_log_directory")).toEqual([{}]);
  });

  it("delivers event payloads until the subscription is dropped", async () => {
    answer(undefined, { events: true });
    const gateway = createTauriCommandGateway();
    const handler = vi.fn();

    const unsubscribe = await gateway.subscribe("workspace:restore-loading", handler);
    await emit("workspace:restore-loading", true);
    expect(handler).toHaveBeenCalledWith(true);

    unsubscribe();
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    await emit("workspace:restore-loading", false);
    warn.mockRestore();

    expect(handler).toHaveBeenCalledTimes(1);
  });
});

describe("Tauri dialog gateway", () => {
  it("asks for one file filtered by extension, or for one directory", async () => {
    answer((cmd) => (cmd === "plugin:dialog|open" ? "/tmp/app.apk" : null));
    const dialog = createTauriDialogGateway();

    await expect(dialog.pickPath({ target: "file", extensions: ["apk"] })).resolves.toBe(
      "/tmp/app.apk",
    );
    await dialog.pickPath({ target: "directory" });

    expect(payloadsOf("plugin:dialog|open")).toEqual([
      {
        options: {
          directory: false,
          multiple: false,
          filters: [{ name: "APK", extensions: ["apk"] }],
        },
      },
      { options: { directory: true, multiple: false, filters: undefined } },
    ]);
  });

  it("treats a cancelled dialog or an unexpected list as no choice", async () => {
    const replies: unknown[] = [null, ["/a", "/b"]];
    answer(() => replies.shift());
    const dialog = createTauriDialogGateway();

    await expect(dialog.pickPath({ target: "file" })).resolves.toBeNull();
    await expect(dialog.pickPath({ target: "file" })).resolves.toBeNull();
  });
});

describe("Tauri log gateway", () => {
  it("writes to the log plugin and never throws when logging itself fails", async () => {
    answer(() => {
      throw new Error("the log plugin is not registered");
    });
    const log = createTauriLogGateway();

    expect(() => log.error("render failed")).not.toThrow();
    await vi.waitFor(() =>
      expect(payloadsOf("plugin:log|log")).toEqual([
        expect.objectContaining({ message: "render failed" }),
      ]),
    );
  });
});

describe("Tauri window gateway", () => {
  it("drives the current window and opens links through the opener plugin", async () => {
    answer((cmd) => (cmd === "plugin:window|is_maximized" ? true : null));
    const windows = createTauriWindowGateway();

    await windows.minimize();
    await expect(windows.isMaximized()).resolves.toBe(true);
    await windows.openExternal("https://developer.android.com/tools/releases/platform-tools");

    expect(payloadsOf("plugin:window|minimize")).toEqual([{ label: "main" }]);
    expect(payloadsOf("plugin:opener|open_url")).toEqual([
      expect.objectContaining({
        url: "https://developer.android.com/tools/releases/platform-tools",
      }),
    ]);
  });

  it("creates a workspace window and focuses it once it exists", async () => {
    answer();
    const windows = createTauriWindowGateway();

    await windows.openWorkspaceWindow({
      label: "workspace-2",
      url: "/",
      title: "Workspace",
      width: 1280,
      height: 800,
    });

    expect(payloadsOf("plugin:webview|create_webview_window")).toEqual([
      {
        options: expect.objectContaining({
          label: "workspace-2",
          url: "/",
          title: "Workspace",
          width: 1280,
          height: 800,
        }),
      },
    ]);
    await vi.waitFor(() =>
      expect(payloadsOf("plugin:window|set_focus")).toEqual([{ label: "workspace-2" }]),
    );
  });
});
