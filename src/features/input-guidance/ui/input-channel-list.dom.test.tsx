import { create, type MessageInitShape } from "@bufbuild/protobuf";
import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { InputChannelsSchema, InputKind, InputService } from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { inputGuidanceMessages } from "../messages";
import { InputChannelList } from "./input-channel-list";

const vendorCopy = {
  "zh-CN": { vendorIme: { label: "厂商输入法", install: "装上厂商输入法" } },
  "en-US": { vendorIme: { label: "Vendor keyboard", install: "Install the vendor keyboard" } },
};

function renderList(channels: MessageInitShape<typeof InputChannelsSchema>) {
  installFakeServices((router) =>
    router.service(InputService, {
      listInputChannels: () => create(InputChannelsSchema, channels),
    }),
  );
  const { wrapper } = createWorkspaceTestHarness(inputGuidanceMessages, vendorCopy);

  render(<InputChannelList />, { wrapper });
}

describe("InputChannelList", () => {
  it("lists the channels in the order they are asked, with what they need", async () => {
    renderList({
      channels: [
        {
          id: "vendor.ime",
          label: { key: "vendorIme.label" },
          kinds: [InputKind.TEXT],
          setup: [{ key: "vendorIme.install" }],
        },
        {
          id: "core.yadb",
          label: { key: "workspace.input.yadb.label" },
          description: { key: "workspace.input.yadb.description" },
          kinds: [InputKind.TEXT, InputKind.CLIPBOARD],
        },
      ],
    });

    const section = await screen.findByRole("region", { name: "Input channels" });
    const [vendor, yadb] = Array.from(within(section).getAllByRole("list")[0]?.children ?? []);

    expect(vendor).toHaveTextContent("Vendor keyboard");
    expect(vendor).toHaveTextContent("Install the vendor keyboard");
    expect(yadb).toHaveTextContent("YADB (built-in)");
    expect(within(section).getAllByText("Input text")).toHaveLength(2);
    expect(within(section).getByText("Write clipboard")).toBeInTheDocument();
  });

  it("warns that text cannot be sent when no channel is installed", async () => {
    renderList({ channels: [] });

    expect(await screen.findByText(/No input channel is installed/)).toBeInTheDocument();
  });
});
