import type { LocalePack } from "../../locale-pack";
import { commonZh } from "./common";
import { enumsZh } from "./enums";
import { navigationZh } from "./navigation";
import { workspaceZh } from "./workspace";

export const zhCN: LocalePack = {
  locale: "zh-CN",
  label: "中文",
  matches: ["zh"],
  messages: {
    ...commonZh,
    ...navigationZh,
    ...enumsZh,
    ...workspaceZh,
  },
};
