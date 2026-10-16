import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { httpEndpoints, procedures } from "./http-endpoints";

const serverFile = path.join(process.cwd(), "src-tauri/src/workspace/api/server.rs");

function routeCalls(source: string): string[] {
  return Array.from(source.matchAll(/\.route\(/g), (match) => {
    const start = (match.index ?? 0) + match[0].length;
    let depth = 1;
    let end = start;

    while (end < source.length && depth > 0) {
      if (source[end] === "(") depth += 1;
      if (source[end] === ")") depth -= 1;
      end += 1;
    }

    return source.slice(start, end - 1);
  });
}

function backendRoutes(): string[] {
  const source = readFileSync(serverFile, "utf8");
  const constants = new Map(
    Array.from(source.matchAll(/const (\w+): &str = "([^"]+)";/g), ([, name, value]) => [
      name,
      value,
    ]),
  );

  return routeCalls(source).flatMap((call) => {
    const [, literal, constant, handlers] = call.match(/^\s*(?:"([^"]+)"|(\w+)),([\s\S]*)$/) ?? [];
    const routePath = literal ?? constants.get(constant) ?? constant;

    return Array.from(
      (handlers ?? "").matchAll(/(?<![\w])(get|post|put|delete|patch)\(/g),
      ([, method]) => `${method.toUpperCase()} ${routePath}`,
    );
  });
}

describe("local API endpoints", () => {
  it("lists exactly the routes the backend serves besides its procedures", () => {
    const listed = httpEndpoints.map((endpoint) => `${endpoint.method} ${endpoint.path}`);

    expect(backendRoutes().length).toBeGreaterThan(0);
    expect([...listed].sort()).toEqual([...backendRoutes()].sort());
  });

  it("lists the procedures the proto contracts open to the local API", () => {
    const paths = procedures.map((procedure) => procedure.path);

    expect(paths).toContain("/yunheshiguang.workspace.v1.WorkspaceService/GetWorkspaceSnapshot");
    expect(paths).not.toContain("/yunheshiguang.api.v1.ApiGatewayService/CreateApiToken");
    expect(procedures.find((procedure) => procedure.id === "get_workspace_snapshot")?.read).toBe(
      true,
    );
    expect(procedures.find((procedure) => procedure.id === "run_batch_action")?.read).toBe(false);
  });
});
