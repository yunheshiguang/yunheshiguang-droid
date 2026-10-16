import {
  type DocumentActions,
  type StoredDocuments,
  useDocumentActions,
  useDocumentList,
  useDocumentListError,
} from "@/shared/api";

import { type AutomationFlow, readAutomationFlow, sortAutomationFlows } from "./flow";
import { encodeFlowDocument } from "./flow-document";

export const AUTOMATION_FLOWS_COLLECTION = "automation-flows";

const NO_FLOWS: readonly AutomationFlow[] = [];

export function toFlowDocumentBody(flow: AutomationFlow): string {
  return encodeFlowDocument(flow);
}

export function readFlowDocuments(list: StoredDocuments): AutomationFlow[] {
  return sortAutomationFlows(
    list.documents.flatMap((document) => {
      const flow = readAutomationFlow(document.id, document.body);

      return flow ? [flow] : [];
    }),
  );
}

export function useAutomationFlows(): readonly AutomationFlow[] {
  return useDocumentList(AUTOMATION_FLOWS_COLLECTION, readFlowDocuments) ?? NO_FLOWS;
}

export function useAutomationFlowsError(): Error | null {
  return useDocumentListError(AUTOMATION_FLOWS_COLLECTION);
}

export type AutomationFlowActions = DocumentActions<AutomationFlow>;

export function useAutomationFlowActions(): AutomationFlowActions {
  return useDocumentActions(AUTOMATION_FLOWS_COLLECTION, toFlowDocumentBody);
}
