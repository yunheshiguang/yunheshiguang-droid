import { Store } from "lucide-react";
import { FlowTemplateList, SectionCard, useI18n } from "yunheshiguang/sdk";

const MARKET_SOURCE_ID = "reference.market";

export function TemplateMarketPage() {
  const { t } = useI18n();

  return (
    <SectionCard icon={Store} title={t("templateMarket.title")}>
      <FlowTemplateList sourceId={MARKET_SOURCE_ID} />
    </SectionCard>
  );
}
