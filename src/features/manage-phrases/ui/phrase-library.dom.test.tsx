import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import {
  createFakeDocuments,
  createWorkspaceTestHarness,
  installFakeServices,
} from "@/shared/testing";

import { managePhrasesMessages } from "../messages";
import { PhraseLibrary } from "./phrase-library";

function renderLibrary() {
  const documents = createFakeDocuments();
  installFakeServices(documents.routes);
  const { wrapper } = createWorkspaceTestHarness(managePhrasesMessages);

  render(<PhraseLibrary />, { wrapper });
  return documents;
}

async function addPhrase(title: string, body: string, tags = "") {
  const user = userEvent.setup();

  await user.click(screen.getByRole("button", { name: "New text snippet" }));
  const editor = screen.getByRole("form", { name: "Edit text snippet" });
  await user.type(within(editor).getByLabelText("Title"), title);
  await user.type(within(editor).getByLabelText("Text"), body);
  if (tags) {
    await user.type(within(editor).getByLabelText("Tags"), tags);
  }
  await user.click(within(editor).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(screen.queryByRole("form")).not.toBeInTheDocument());

  return user;
}

describe("PhraseLibrary", () => {
  it("asks for a title and text before saving a phrase as a document", async () => {
    const user = userEvent.setup();
    const documents = renderLibrary();

    expect(await screen.findByText(/No text snippets yet/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "New text snippet" }));
    const editor = screen.getByRole("form", { name: "Edit text snippet" });
    await user.click(within(editor).getByRole("button", { name: "Save" }));

    expect(within(editor).getByText("Enter a title")).toBeInTheDocument();
    expect(within(editor).getByText("Enter text")).toBeInTheDocument();
    expect(documents.bodies("phrases")).toEqual([]);

    await user.click(within(editor).getByRole("button", { name: "Close" }));
    await addPhrase("Welcome", "你好，欢迎光临！", "support, TikTok, tiktok");

    expect(documents.bodies("phrases")).toEqual([
      { version: 1, title: "Welcome", body: "你好，欢迎光临！", tags: ["support", "TikTok"] },
    ]);
    const list = await screen.findByRole("list", { name: "Text snippets" });
    expect(within(list).getByText("你好，欢迎光临！")).toBeInTheDocument();
  });

  it("finds phrases by text and by tag", async () => {
    renderLibrary();
    await screen.findByText(/No text snippets yet/);
    const user = await addPhrase("Welcome", "Hello there", "support");
    await addPhrase("Payment", "Your order is waiting", "orders");

    await user.selectOptions(screen.getByLabelText("Filter by tag"), "orders");
    const list = screen.getByRole("list", { name: "Text snippets" });
    expect(within(list).queryByText("Welcome")).not.toBeInTheDocument();
    expect(within(list).getByText("Payment")).toBeInTheDocument();

    await user.selectOptions(screen.getByLabelText("Filter by tag"), "all");
    await user.type(screen.getByLabelText("Search titles, text, or tags"), "nothing like it");
    expect(screen.getByText("No text snippets match")).toBeInTheDocument();
  });

  it("edits and deletes a stored phrase", async () => {
    const documents = renderLibrary();
    await screen.findByText(/No text snippets yet/);
    const user = await addPhrase("Welcome", "Hello");

    await user.click(screen.getByRole("button", { name: "Edit “Welcome”" }));
    const editor = screen.getByRole("form", { name: "Edit text snippet" });
    await user.clear(within(editor).getByLabelText("Text"));
    await user.type(within(editor).getByLabelText("Text"), "Hi again");
    await user.click(within(editor).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(documents.bodies("phrases")[0]).toMatchObject({ body: "Hi again" }));

    await user.click(await screen.findByRole("button", { name: "Edit “Welcome”" }));
    await user.click(screen.getByRole("button", { name: "Delete" }));
    await user.click(
      within(await screen.findByRole("alertdialog")).getByRole("button", { name: "Delete" }),
    );
    expect(await screen.findByText(/No text snippets yet/)).toBeInTheDocument();
    expect(documents.bodies("phrases")).toEqual([]);
  });
});
