import { defineWorkspaceExtension } from "yunheshiguang/sdk";

import { enterpriseModule } from "./enterprise";
import { referenceMessages } from "./messages";
import { templateMarketModule } from "./template-market";

export default defineWorkspaceExtension({
  id: "reference",
  modules: [templateMarketModule, enterpriseModule],
  messages: referenceMessages,
});
