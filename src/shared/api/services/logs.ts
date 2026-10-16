import { commandGateway } from "../gateway";
import { WORKSPACE_COMMANDS } from "../workspace-commands";

export async function revealLogDirectory(): Promise<void> {
  await commandGateway().invoke<void>(WORKSPACE_COMMANDS.revealLogDirectory);
}
