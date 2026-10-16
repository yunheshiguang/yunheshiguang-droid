import type { LocalePack } from "../../locale-pack";
import { commonEn } from "./common";
import { enumsEn } from "./enums";
import { navigationEn } from "./navigation";
import { workspaceEn } from "./workspace";

export const enUS: LocalePack = {
  locale: "en-US",
  label: "English",
  matches: ["en"],
  messages: {
    ...commonEn,
    ...navigationEn,
    ...enumsEn,
    ...workspaceEn,
  },
};
