import type { ComponentProps } from "react";

import { DiscordBridgeView } from "../discord/DiscordBridgeView";
import { GeneralSettingsSections } from "../settings/GeneralSettingsSections";
import { RulesLauncherCard } from "../settings/RulesLauncherCard";
import { SettingsFlowView } from "../settings/SettingsFlowView";

export function SettingsPage({
  discordBridgeProps,
  rulesLauncherProps,
  generalSettingsProps,
}: {
  discordBridgeProps: ComponentProps<typeof DiscordBridgeView>;
  rulesLauncherProps: ComponentProps<typeof RulesLauncherCard>;
  generalSettingsProps: ComponentProps<typeof GeneralSettingsSections>;
}) {
  return (
    <SettingsFlowView
      stepOne={<DiscordBridgeView {...discordBridgeProps} />}
      stepTwo={<RulesLauncherCard {...rulesLauncherProps} />}
      stepThree={<GeneralSettingsSections {...generalSettingsProps} />}
    />
  );
}
