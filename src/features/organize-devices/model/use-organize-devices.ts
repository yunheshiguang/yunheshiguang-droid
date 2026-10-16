import type { MessageInitShape } from "@bufbuild/protobuf";
import { useMutation } from "@connectrpc/connect-query";
import { useIsMutating, useQueryClient } from "@tanstack/react-query";

import { snapshotKey, workspaceMutationKeys } from "@/shared/api";
import {
  type DeviceGroupDefinitionSchema,
  type DeviceProfilePatchSchema,
  type OrganizeResult,
  ProfileService,
} from "@/shared/proto";

export type DeviceProfilePatch = MessageInitShape<typeof DeviceProfilePatchSchema>;

type DeviceGroupDefinition = MessageInitShape<typeof DeviceGroupDefinitionSchema>;

export type OrganizeActions = {
  update: (deviceIds: readonly string[], patch: DeviceProfilePatch) => Promise<OrganizeResult>;
  saveGroup: (group: DeviceGroupDefinition) => Promise<OrganizeResult>;
  removeGroup: (id: string) => Promise<OrganizeResult>;
  pending: boolean;
};

export function useOrganizeDevices(): OrganizeActions {
  const queryClient = useQueryClient();
  const mutationKey = workspaceMutationKeys.organize();
  const options = {
    mutationKey,
    onSettled: () => queryClient.invalidateQueries({ queryKey: snapshotKey() }),
  };
  const update = useMutation(ProfileService.method.updateDeviceProfiles, options);
  const save = useMutation(ProfileService.method.saveDeviceGroup, options);
  const remove = useMutation(ProfileService.method.removeDeviceGroup, options);
  const pending = useIsMutating({ mutationKey }) > 0;

  return {
    update: (deviceIds, patch) => update.mutateAsync({ deviceIds: [...deviceIds], patch }),
    saveGroup: (group) => save.mutateAsync({ group }),
    removeGroup: (id) => remove.mutateAsync({ id }),
    pending,
  };
}
