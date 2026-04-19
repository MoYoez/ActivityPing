import type { ComponentProps } from "react";

import {
  RuntimeDebugCard,
  RuntimeLogCard,
  RuntimeMonitorCard,
  RuntimePrerequisiteCard,
} from "../runtime/RuntimeCards";

export function RuntimePage({
  prerequisiteCardProps,
  monitorCardProps,
  logCardProps,
  debugCardProps,
}: {
  prerequisiteCardProps?: ComponentProps<typeof RuntimePrerequisiteCard> | null;
  monitorCardProps: ComponentProps<typeof RuntimeMonitorCard>;
  logCardProps: ComponentProps<typeof RuntimeLogCard>;
  debugCardProps: ComponentProps<typeof RuntimeDebugCard>;
}) {
  return (
    <div className="runtime-layout">
      {prerequisiteCardProps ? <RuntimePrerequisiteCard {...prerequisiteCardProps} /> : null}
      <RuntimeMonitorCard {...monitorCardProps} />
      <RuntimeLogCard {...logCardProps} />
      <RuntimeDebugCard {...debugCardProps} />
    </div>
  );
}
