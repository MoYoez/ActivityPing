import type { ComponentProps } from "react";

import { RuleGroupsEditorSection } from "./RuleGroupsEditorSection";
import { RuleSupportSections } from "./RuleSupportSections";

export function RulesDialogContent({
  ruleGroupsSectionProps,
  ruleSupportSectionProps,
}: {
  ruleGroupsSectionProps: ComponentProps<typeof RuleGroupsEditorSection>;
  ruleSupportSectionProps: ComponentProps<typeof RuleSupportSections>;
}) {
  return (
    <>
      <RuleGroupsEditorSection {...ruleGroupsSectionProps} />
      <RuleSupportSections {...ruleSupportSectionProps} />
    </>
  );
}
