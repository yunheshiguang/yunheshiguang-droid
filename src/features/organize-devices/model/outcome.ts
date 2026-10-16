import { useState } from "react";

import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { textValue } from "@/shared/lib/workspace-view";
import type { OrganizeResult } from "@/shared/proto";

export function useOrganizeOutcome() {
  const { t } = useI18n();
  const [problem, setProblem] = useState<string | null>(null);

  const track = async (operation: Promise<OrganizeResult>): Promise<boolean> => {
    setProblem(null);

    try {
      const result = await operation;

      if (!result.accepted) {
        setProblem(textValue(result.detail, true, t) || t("organizeDevices.refused"));
      }
      return result.accepted;
    } catch (error) {
      setProblem(errorMessage(error));
      return false;
    }
  };

  return { problem, track };
}
