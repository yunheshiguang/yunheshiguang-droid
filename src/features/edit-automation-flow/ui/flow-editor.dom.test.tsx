import { Code, ConnectError, type ServiceImpl } from "@connectrpc/connect";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { assistantMessages } from "@/entities/assistant";
import { taskMessages } from "@/entities/task";
import { DocumentService } from "@/shared/proto";
import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { editAutomationFlowMessages } from "../messages";
import { FlowEditor } from "./flow-editor";

function renderEditor(
  overrides: Partial<ServiceImpl<typeof DocumentService>> = {},
  seed: (documents: ReturnType<typeof createFakeDocuments>) => void = () => {},
) {
  const documents = createFakeDocuments();
  seed(documents);
  installFakeServices((router) =>
    router.service(DocumentService, { ...documents.service, ...overrides }),
  );
  const { wrapper } = createWorkspaceTestHarness(
    taskMessages,
    editAutomationFlowMessages,
    assistantMessages,
  );

  render(<FlowEditor />, { wrapper });
  return documents;
}

async function draftFlow(name: string) {
  const user = userEvent.setup();

  await user.click(screen.getByRole("button", { name: "New flow" }));
  const editor = screen.getByRole("form", { name: "Edit flow" });
  await user.type(within(editor).getByLabelText("Flow name"), name);
  await user.click(within(editor).getByRole("button", { name: "Add step" }));
  await user.selectOptions(within(editor).getByLabelText("Step 1"), "RECOVER_CONNECTION");
  await user.click(within(editor).getByRole("button", { name: "Save flow" }));

  return { user, editor };
}

describe("FlowEditor", () => {
  it("says when saved flows cannot be read instead of showing an empty library", async () => {
    renderEditor({
      listDocuments: () => {
        throw new ConnectError("the synced folder is offline", Code.Unavailable);
      },
    });

    expect(
      await screen.findByText("Could not read what was saved here", {}, { timeout: 5000 }),
    ).toBeInTheDocument();
    expect(screen.getByText(/the synced folder is offline/)).toBeInTheDocument();
  });

  it("builds, validates and saves a flow as a backend document", async () => {
    const user = userEvent.setup();
    const documents = renderEditor();

    expect(await screen.findByText("No flows yet")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "New flow" }));
    const editor = screen.getByRole("form", { name: "Edit flow" });

    await user.click(within(editor).getByRole("button", { name: "Save flow" }));
    expect(within(editor).getByText("Enter a flow name")).toBeInTheDocument();
    expect(within(editor).getByText("Add at least one step")).toBeInTheDocument();

    await user.type(within(editor).getByLabelText("Flow name"), "Install and launch");
    await user.click(within(editor).getByRole("button", { name: "Add step" }));
    await user.selectOptions(within(editor).getByLabelText("Step 1"), "LAUNCH_APP");
    await user.click(within(editor).getByRole("button", { name: "Add step" }));
    await user.selectOptions(within(editor).getByLabelText("Step 2"), "INSTALL_APK");

    await user.click(within(editor).getByRole("button", { name: "Save flow" }));
    expect(within(editor).getAllByText("Enter a value for this step")).toHaveLength(2);

    await user.type(within(editor).getByLabelText("Value for step 1"), "com.example");
    await user.type(within(editor).getByLabelText("Value for step 2"), "/tmp/app.apk");
    await user.click(within(editor).getByRole("button", { name: "Move step 2 up" }));
    await user.click(within(editor).getByRole("button", { name: "Save flow" }));

    expect(await within(editor).findByRole("status")).toHaveTextContent("Saved");
    expect(await screen.findByRole("button", { name: "Install and launch" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(documents.bodies("automation-flows")).toEqual([
      {
        name: "Install and launch",
        steps: [
          expect.objectContaining({
            kind: "BATCH_ACTION_KIND_INSTALL_APK",
            value: "/tmp/app.apk",
          }),
          expect.objectContaining({ kind: "BATCH_ACTION_KIND_LAUNCH_APP", value: "com.example" }),
        ],
      },
    ]);
  });

  it("shows a step this version does not know and will not save it as something else", async () => {
    const user = userEvent.setup();
    const future = {
      name: "Future",
      steps: [{ id: "a", kind: "BATCH_ACTION_KIND_TELEPORT", value: "" }],
    };
    const documents = renderEditor({}, (seeded) =>
      seeded.save("automation-flows", "future", JSON.stringify(future)),
    );

    await user.click(await screen.findByRole("button", { name: "Future" }));
    const editor = screen.getByRole("form", { name: "Edit flow" });
    expect(within(editor).getByLabelText("Step 1")).toHaveDisplayValue(
      "An action this version does not offer",
    );
    expect(
      within(editor).getByText(
        "This version cannot run this step. Choose another action or remove it",
      ),
    ).toBeInTheDocument();

    await user.click(within(editor).getByRole("button", { name: "Save flow" }));
    expect(within(editor).queryByRole("status")).not.toBeInTheDocument();
    expect(documents.bodies("automation-flows")).toEqual([future]);
  });

  it("deletes a saved flow", async () => {
    const documents = renderEditor();
    const { user, editor } = await draftFlow("Recover");

    await screen.findByRole("button", { name: "Recover" });
    await user.click(within(editor).getByRole("button", { name: "Delete flow" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "Delete flow" }),
    );

    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Recover" })).not.toBeInTheDocument(),
    );
    expect(screen.getByText("No flows yet")).toBeInTheDocument();
    expect(documents.bodies("automation-flows")).toEqual([]);
  });

  it("keeps the draft open and says so when the backend refuses to save it", async () => {
    renderEditor({ saveDocument: () => Promise.reject(new Error("disk full")) });
    const { editor } = await draftFlow("Recover");

    expect(
      await within(editor).findByText("Could not update the flow library"),
    ).toBeInTheDocument();
    expect(within(editor).getByLabelText("Flow name")).toHaveValue("Recover");
    expect(within(editor).queryByRole("status")).not.toBeInTheDocument();
  });
});
