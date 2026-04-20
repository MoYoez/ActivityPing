import type { ComponentProps } from "react";

import { DiscordCustomAssetLibrary } from "../discord/DiscordCustomAssetLibrary";

export function ResourcesPage({
  assetLibraryProps,
}: {
  assetLibraryProps: ComponentProps<typeof DiscordCustomAssetLibrary>;
}) {
  return (
    <div className="settings-flow">
      <DiscordCustomAssetLibrary {...assetLibraryProps} />
    </div>
  );
}
