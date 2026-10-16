import { create } from "@bufbuild/protobuf";
import { useQuery } from "@connectrpc/connect-query";

import { errorMessage } from "@/shared/lib/error-message";
import {
  type FlowTemplateCatalog,
  FlowTemplateCatalogSchema,
  TemplateService,
} from "@/shared/proto";

const EMPTY: FlowTemplateCatalog = create(FlowTemplateCatalogSchema);

export type FlowTemplates = {
  catalog: FlowTemplateCatalog;
  loading: boolean;
  error: string | null;
};

export function useFlowTemplates(sourceId?: string): FlowTemplates {
  const { data, isPending, error } = useQuery(
    TemplateService.method.listFlowTemplates,
    {},
    { staleTime: Number.POSITIVE_INFINITY },
  );
  const catalog = data ?? EMPTY;

  return {
    catalog: sourceId
      ? {
          ...catalog,
          sources: catalog.sources.filter((source) => source.id === sourceId),
          templates: catalog.templates.filter((template) => template.sourceId === sourceId),
        }
      : catalog,
    loading: isPending,
    error: error == null ? null : errorMessage(error),
  };
}
