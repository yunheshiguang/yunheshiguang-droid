import type { MessageInitShape } from "@bufbuild/protobuf";
import {
  callUnaryMethod,
  useQuery,
  useMutation as useRpcMutation,
  useTransport,
} from "@connectrpc/connect-query";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { dialogGateway, documentsKey, workspaceMutationKeys } from "@/shared/api";
import {
  type ConflictPolicy,
  type MarketPackage,
  type PackageManifestSchema,
  type PackageOriginSchema,
  PackageService,
} from "@/shared/proto";

const PACKAGE_EXTENSION = "json";
const NO_PACKAGES: readonly MarketPackage[] = [];

export type PackageOrigin = MessageInitShape<typeof PackageOriginSchema>;

export type PackageManifestFields = Required<
  Pick<
    MessageInitShape<typeof PackageManifestSchema>,
    "id" | "name" | "version" | "author" | "description"
  >
>;

export const fileOrigin = (path: string): PackageOrigin => ({
  origin: { case: "path", value: path },
});

export const marketOrigin = (sourceId: string, packageId: string): PackageOrigin => ({
  origin: { case: "market", value: { sourceId, packageId } },
});

export function useMarketPackages(): readonly MarketPackage[] {
  const { data } = useQuery(
    PackageService.method.listMarketPackages,
    {},
    { staleTime: Number.POSITIVE_INFINITY },
  );

  return data?.packages ?? NO_PACKAGES;
}

export function useExportPackage() {
  const transport = useTransport();
  const mutation = useMutation({
    mutationKey: workspaceMutationKeys.packages(),
    mutationFn: async ({
      manifest,
      flowIds,
    }: {
      manifest: PackageManifestFields;
      flowIds: readonly string[];
    }) => {
      const path = await dialogGateway().pickPath({
        target: "save",
        suggestedName: `${manifest.id || "package"}-${manifest.version || "1.0.0"}.${PACKAGE_EXTENSION}`,
        extensions: [PACKAGE_EXTENSION],
      });

      return path
        ? callUnaryMethod(transport, PackageService.method.exportPackage, {
            manifest,
            flowIds: [...flowIds],
            path,
          })
        : undefined;
    },
  });

  return { exportAs: mutation.mutateAsync, exporting: mutation.isPending };
}

export function pickPackageFile(): Promise<string | null> {
  return dialogGateway().pickPath({ target: "file", extensions: [PACKAGE_EXTENSION] });
}

export function useInspectPackage() {
  const mutation = useRpcMutation(PackageService.method.inspectPackage);

  return {
    inspect: (origin: PackageOrigin) => mutation.mutateAsync({ origin }),
    inspecting: mutation.isPending,
  };
}

export function useTrustPackageSigner() {
  const mutation = useRpcMutation(PackageService.method.trustPackageSigner, {
    mutationKey: workspaceMutationKeys.packages(),
  });

  return { trust: mutation.mutateAsync, trusting: mutation.isPending };
}

export function useInstallPackage() {
  const queryClient = useQueryClient();
  const mutation = useRpcMutation(PackageService.method.installPackage, {
    mutationKey: workspaceMutationKeys.packages(),
    onSuccess: () =>
      Promise.all(
        ["automation-flows", "scripts"].map((collection) =>
          queryClient.invalidateQueries({ queryKey: documentsKey(collection) }),
        ),
      ),
  });

  return {
    install: ({ origin, policy }: { origin: PackageOrigin; policy: ConflictPolicy }) =>
      mutation.mutateAsync({ origin, onConflict: policy }),
    installing: mutation.isPending,
  };
}
