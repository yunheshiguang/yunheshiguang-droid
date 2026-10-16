import { create } from "@bufbuild/protobuf";
import type { ServiceImpl } from "@connectrpc/connect";

import { type CloudPhone, CloudPhoneSchema, type CloudService } from "@/shared/proto";

import type { MockContext } from "./mock-context";

const REGISTERED = "core.registered";

export function createMockCloud(context: MockContext): ServiceImpl<typeof CloudService> {
  const phones: CloudPhone[] = [];
  const refused = (key: string, params?: Record<string, string>) =>
    context.settle({ accepted: false, detail: { key, params } });
  const unsupported = () => refused("workspace.cloud.unsupported");
  const target = (phone: CloudPhone) =>
    `${phone.endpoint?.host ?? ""}:${phone.endpoint?.port ?? 0}`;

  return {
    listCloudPhones: () =>
      context.settle({
        providers: [
          {
            id: REGISTERED,
            label: { key: "workspace.cloud.registered.label" },
            description: { key: "workspace.cloud.registered.description" },
            canRegister: true,
          },
        ],
        phones: [...phones],
      }),
    startCloudPhone: unsupported,
    stopCloudPhone: unsupported,
    registerCloudPhone: (request) => {
      const name = request.name.trim();
      const host = request.endpoint?.host.trim() ?? "";
      const port = request.endpoint?.port ?? 0;

      if (!name) {
        return refused("workspace.cloud.invalid.name", { max: "80" });
      }
      if (!host || /\s/.test(host) || port < 1 || port > 65_535) {
        return refused("workspace.cloud.invalid.endpoint");
      }
      if (phones.some((phone) => target(phone) === `${host}:${port}`)) {
        return refused("workspace.cloud.alreadyRegistered");
      }
      const phone = create(CloudPhoneSchema, {
        providerId: REGISTERED,
        id: crypto.randomUUID(),
        name,
        endpoint: { host, port },
      });
      phones.push(phone);
      context.audit("GOVERNED_ACTION_MANAGE_CLOUD_PHONES", true, [target(phone)], {
        key: "workspace.cloud.audit.registered",
        params: { name },
      });
      return context.settle({ accepted: true, phone });
    },
    removeCloudPhone: (request) => {
      const index = phones.findIndex((phone) => phone.id === request.phoneId);

      if (request.providerId !== REGISTERED) {
        return unsupported();
      }
      if (index < 0) {
        return refused("workspace.cloud.phoneMissing");
      }
      const [phone] = phones.splice(index, 1);
      context.audit("GOVERNED_ACTION_MANAGE_CLOUD_PHONES", true, [target(phone)], {
        key: "workspace.cloud.audit.removed",
        params: { name: phone.name },
      });
      return context.settle({ accepted: true, phone });
    },
  };
}
