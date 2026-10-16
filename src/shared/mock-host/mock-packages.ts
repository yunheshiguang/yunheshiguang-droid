import { toJson } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import {
  ConflictPolicy,
  type FlowJson,
  type PackageManifestJson,
  PackageManifestSchema,
  type PackageOrigin,
  type PackageService,
  type ScriptJson,
  type TemplatePackageJson,
} from "@/shared/proto";

import { fromMockJson, type MockContext } from "./mock-context";

const MOCK_KEY_ID = "5D3A1F0C9B8E7D6A";
const MOCK_PUBLIC_KEY = "RWRqfY7rnAkfO11mock0signer0public0key0for0the0browser0preview";
const FLOWS = "automation-flows";
const SCRIPTS = "scripts";
const SEMVER = /^\d+\.\d+\.\d+(?:-[\w.]+)?(?:\+[\w.]+)?$/;
const PACKAGE_ID = /^[A-Za-z0-9][A-Za-z0-9._-]{0,99}$/;

export function createMockPackages(context: MockContext): ServiceImpl<typeof PackageService> {
  const files = new Map<string, TemplatePackageJson>();
  const trusted = new Set<string>();

  const library = <T>(collection: string) =>
    new Map(
      (context.documents.list({ collection }).documents ?? []).map((document) => [
        document.id ?? "",
        JSON.parse(document.body ?? "{}") as T,
      ]),
    );
  const load = (origin?: PackageOrigin) =>
    origin?.origin.case === "path" ? files.get(origin.origin.value) : undefined;
  const manifestOf = (manifest?: PackageManifestJson) =>
    manifest ? fromMockJson(PackageManifestSchema, manifest) : undefined;
  const unreadable = () =>
    context.settle({
      accepted: false,
      detail: { key: "workspace.packages.readFailed", params: { message: "not found" } },
    });

  return {
    exportPackage: (request) => {
      const manifest: PackageManifestJson = request.manifest
        ? toJson(PackageManifestSchema, request.manifest)
        : {};
      const refuse = (key: string) => context.settle({ accepted: false, detail: { key } });

      if (!PACKAGE_ID.test(manifest.id ?? "")) return refuse("workspace.packages.invalid.id");
      if (!manifest.name?.trim()) return refuse("workspace.packages.invalid.nameRequired");
      if (!SEMVER.test(manifest.version ?? "")) return refuse("workspace.packages.invalid.version");
      if (!request.flowIds.length) return refuse("workspace.packages.invalid.noFlows");
      if (!request.path.startsWith("/")) return refuse("workspace.packages.invalid.path");

      const flows = library<FlowJson>(FLOWS);
      const scripts = library<ScriptJson>(SCRIPTS);
      const chosen = request.flowIds.map((id) => ({ id, flow: flows.get(id) }));
      const scriptIds = new Set(
        chosen.flatMap(({ flow }) =>
          (flow?.steps ?? [])
            .filter((step) => step.kind === "BATCH_ACTION_KIND_RUN_SCRIPT" && step.value)
            .map((step) => step.value ?? ""),
        ),
      );
      files.set(request.path, {
        manifest: { ...manifest, createdAt: new Date().toISOString() },
        flows: chosen,
        scripts: [...scriptIds].map((id) => ({ id, script: scripts.get(id) })),
      });
      context.audit("GOVERNED_ACTION_MANAGE_PACKAGES", true, [], {
        key: "workspace.packages.audit.exported",
        params: { name: manifest.name, version: manifest.version ?? "" },
      });
      return context.settle({
        accepted: true,
        detail: {
          key: "workspace.packages.exported",
          params: {
            flows: String(chosen.length),
            scripts: String(scriptIds.size),
            path: request.path,
          },
        },
        flows: chosen.length,
        scripts: scriptIds.size,
      });
    },
    inspectPackage: (request) => {
      const found = load(request.origin);

      if (!found) {
        return unreadable();
      }
      const flows = library<FlowJson>(FLOWS);
      const scripts = library<ScriptJson>(SCRIPTS);
      return context.settle({
        accepted: true,
        preview: {
          manifest: manifestOf(found.manifest),
          flows: (found.flows ?? []).map((item) => ({
            id: item.id,
            name: item.flow?.name,
            exists: flows.has(item.id ?? ""),
          })),
          scripts: (found.scripts ?? []).map((item) => ({
            id: item.id,
            name: item.script?.name,
            exists: scripts.has(item.id ?? ""),
          })),
          problems: [],
          verified: true,
          signer: {
            keyId: MOCK_KEY_ID,
            name: found.manifest?.author || "mock",
            trusted: trusted.has(MOCK_PUBLIC_KEY),
            publicKey: MOCK_PUBLIC_KEY,
          },
        },
      });
    },
    trustPackageSigner: (request) => {
      const name = request.name;

      trusted.add(request.publicKey);
      context.audit("GOVERNED_ACTION_MANAGE_PACKAGES", true, [], {
        key: "workspace.packages.audit.signerTrusted",
        params: { name },
      });
      return context.settle({
        accepted: true,
        detail: { key: "workspace.packages.signerTrusted", params: { name } },
      });
    },
    installPackage: (request) => {
      const found = load(request.origin);

      if (!found) {
        return unreadable();
      }
      const policy = request.onConflict;
      let written = { flows: 0, scripts: 0 };
      const place = (collection: string, id: string, body: unknown, kind: "flows" | "scripts") => {
        const taken = library(collection);
        if (taken.has(id) && policy === ConflictPolicy.SKIP) return;
        let target = id;
        for (let n = 2; taken.has(target) && policy !== ConflictPolicy.REPLACE; n += 1) {
          target = `${id}-${n}`;
        }
        context.documents.save({
          collection,
          document: { id: target, body: JSON.stringify(body) },
        });
        written = { ...written, [kind]: written[kind] + 1 };
      };
      for (const item of found.scripts ?? []) place(SCRIPTS, item.id ?? "", item.script, "scripts");
      for (const item of found.flows ?? []) place(FLOWS, item.id ?? "", item.flow, "flows");
      context.audit("GOVERNED_ACTION_MANAGE_PACKAGES", true, [], {
        key: "workspace.packages.audit.installed",
        params: { name: found.manifest?.name ?? "", version: found.manifest?.version ?? "" },
      });
      return context.settle({
        accepted: true,
        detail: {
          key: "workspace.packages.installed",
          params: { flows: String(written.flows), scripts: String(written.scripts) },
        },
        ...written,
      });
    },
    listMarketPackages: () => context.settle({ packages: [] }),
  };
}
