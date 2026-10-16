import { create, fromJson, toJson } from "@bufbuild/protobuf";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";

import type { DeviceRecord } from "@/entities/device";
import type { DeviceGroup } from "@/entities/group";
import type { OrganizeResultJson } from "@/shared/proto";
import {
  ConnectionHealth,
  ConnectionType,
  DeviceGroupSchema,
  DeviceLabelsSchema,
  DeviceRecordSchema,
  DeviceStatus,
  DiagnosticCode,
  GroupTone,
  OrganizeResultSchema,
  PiiClass,
  ProfileService,
  RemoveDeviceGroupRequestSchema,
  SaveDeviceGroupRequestSchema,
  SearchableTextSchema,
  UiTextSchema,
  UpdateDeviceProfilesRequestSchema,
  VisibilityMode,
} from "@/shared/proto";
import { createWorkspaceTestHarness, installFakeServices } from "@/shared/testing";

import { organizeDevicesMessages } from "../messages";
import { DeviceProfileButton, DeviceProfileDialog } from "./device-profile-dialog";
import { GroupManager } from "./group-manager";
import { OrganizeSelectionBar } from "./organize-selection-bar";

const phone: DeviceRecord = create(DeviceRecordSchema, {
  id: "R58M20ABCDE",
  status: DeviceStatus.ONLINE,
  connection: ConnectionType.USB,
  tags: [],
  health: ConnectionHealth.READY,
  diagnostic: DiagnosticCode.NONE,
  selectedRank: 1,
  groupId: "ungrouped",
  labels: create(DeviceLabelsSchema, {
    number: 3,
    alias: "A-03",
    note: create(SearchableTextSchema, {
      displayValue: "old note",
      canonicalValue: "old note",
      aliases: [],
      redactedValue: "******",
      piiClass: PiiClass.NOTE,
      visibility: VisibilityMode.SEARCHABLE,
      searchable: true,
    }),
    tags: ["美区"],
  }),
});

const groups: DeviceGroup[] = [
  create(DeviceGroupSchema, {
    id: "shelf-a",
    name: create(UiTextSchema, { key: "", params: {}, fallback: "Shelf A" }),
    total: 0,
    online: 0,
    tone: GroupTone.SKY,
  }),
  create(DeviceGroupSchema, {
    id: "ungrouped",
    name: create(UiTextSchema, { key: "workspace.device.group.default", params: {}, fallback: "" }),
    total: 1,
    online: 1,
    tone: GroupTone.SLATE,
  }),
];

function recordOrganizing(answer: OrganizeResultJson = { accepted: true, updated: 1 }) {
  const requests: Array<{ command: string; request: unknown }> = [];
  const reply = () => fromJson(OrganizeResultSchema, answer);

  installFakeServices((router) =>
    router.service(ProfileService, {
      updateDeviceProfiles: (request) => {
        requests.push({
          command: "update",
          request: toJson(UpdateDeviceProfilesRequestSchema, request),
        });
        return reply();
      },
      saveDeviceGroup: (request) => {
        requests.push({ command: "save", request: toJson(SaveDeviceGroupRequestSchema, request) });
        return reply();
      },
      removeDeviceGroup: (request) => {
        requests.push({
          command: "remove",
          request: toJson(RemoveDeviceGroupRequestSchema, request),
        });
        return reply();
      },
    }),
  );
  return requests;
}

function harness() {
  return createWorkspaceTestHarness(organizeDevicesMessages).wrapper;
}

function ProfileEditor() {
  const [editing, setEditing] = useState<DeviceRecord | null>(null);

  return (
    <>
      <DeviceProfileButton title="A-03" onOpen={() => setEditing(phone)} />
      <DeviceProfileDialog device={editing} groups={groups} onClose={() => setEditing(null)} />
    </>
  );
}

describe("DeviceProfileDialog", () => {
  it("saves one device's profile, replacing its tags", async () => {
    const user = userEvent.setup();
    const requests = recordOrganizing();
    render(<ProfileEditor />, { wrapper: harness() });

    await user.click(screen.getByRole("button", { name: "Edit the profile of “A-03”" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByLabelText("Note")).toHaveValue("old note");

    await user.clear(within(dialog).getByLabelText("Name"));
    await user.type(within(dialog).getByLabelText("Name"), "A-04");
    await user.selectOptions(within(dialog).getByLabelText("Group"), "shelf-a");
    await user.clear(within(dialog).getByLabelText("Tags"));
    await user.type(within(dialog).getByLabelText("Tags"), "TikTok");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));

    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(requests).toEqual([
      {
        command: "update",
        request: {
          deviceIds: ["R58M20ABCDE"],
          patch: {
            number: 3,
            alias: "A-04",
            note: "old note",
            groupId: "shelf-a",
            addTags: ["TikTok"],
            removeTags: ["美区"],
          },
        },
      },
    ]);
  });

  it("keeps the dialog open and says why the backend refused the change", async () => {
    const user = userEvent.setup();
    recordOrganizing({
      accepted: false,
      detail: { key: "workspace.organize.invalid.aliasTooLong", params: { max: "32" } },
    });
    render(<ProfileEditor />, { wrapper: harness() });

    await user.click(screen.getByRole("button", { name: "Edit the profile of “A-03”" }));
    await user.click(
      within(await screen.findByRole("dialog")).getByRole("button", { name: "Save" }),
    );

    expect(await screen.findByText("Name can be at most 32 characters")).toBeInTheDocument();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });
});

describe("OrganizeSelectionBar", () => {
  it("stays out of the way until devices are selected", () => {
    recordOrganizing();
    const { container } = render(<OrganizeSelectionBar deviceIds={[]} groups={groups} />, {
      wrapper: harness(),
    });

    expect(container).toBeEmptyDOMElement();
  });

  it("numbers the selection in wall order and moves it into a group", async () => {
    const user = userEvent.setup();
    const requests = recordOrganizing();
    render(<OrganizeSelectionBar deviceIds={["b", "a"]} groups={groups} />, {
      wrapper: harness(),
    });

    await user.clear(screen.getByLabelText("Start at"));
    await user.type(screen.getByLabelText("Start at"), "7");
    await user.click(screen.getByRole("button", { name: "Number in order" }));
    expect(await screen.findByRole("status")).toHaveTextContent("Numbered in device wall order");

    await user.selectOptions(screen.getByLabelText("Move to group"), "shelf-a");
    await waitFor(() => expect(requests).toHaveLength(2));

    expect(requests.map(({ request }) => request)).toEqual([
      { deviceIds: ["b", "a"], patch: { number: 7 } },
      { deviceIds: ["b", "a"], patch: { groupId: "shelf-a" } },
    ]);
  });
});

describe("GroupManager", () => {
  it("creates groups and deletes the ones the operator made", async () => {
    const user = userEvent.setup();
    const requests = recordOrganizing();
    render(<GroupManager groups={groups} />, { wrapper: harness() });

    await user.click(screen.getByRole("button", { name: "Manage groups" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(1);

    await user.type(within(dialog).getByLabelText("New group name"), "Shelf B");
    await user.click(within(dialog).getByRole("button", { name: "Create" }));
    await user.click(within(dialog).getByRole("button", { name: "Delete “Shelf A”" }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm).toHaveTextContent("Delete the group “Shelf A”?");
    await user.click(within(confirm).getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(requests).toHaveLength(2));

    const [created, removed] = requests;
    expect(created).toMatchObject({
      command: "save",
      request: { group: { name: "Shelf B", tone: "GROUP_TONE_EMERALD" } },
    });
    const group = (created?.request as { group?: { id?: string } } | undefined)?.group;
    expect(group?.id).toMatch(/^[0-9a-f-]{36}$/);
    expect(removed).toMatchObject({ command: "remove", request: { id: "shelf-a" } });
  });
});
