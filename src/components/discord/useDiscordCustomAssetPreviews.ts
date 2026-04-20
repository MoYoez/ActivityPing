import { useEffect, useState } from "react";

import { getDiscordCustomAssetPreview } from "../../lib/api";
import type { DiscordCustomAsset } from "../../types";

export function useDiscordCustomAssetPreviews(assets: DiscordCustomAsset[]) {
  const [previewMap, setPreviewMap] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let disposed = false;

    void (async () => {
      setLoading(true);

      if (assets.length === 0) {
        setPreviewMap({});
        setLoading(false);
        return;
      }

      const entries = await Promise.all(
        assets.map(async (asset) => {
          try {
            const previewUrl = await getDiscordCustomAssetPreview(asset.id);
            return [asset.id, previewUrl] as const;
          } catch {
            return [asset.id, ""] as const;
          }
        }),
      );

      if (disposed) {
        return;
      }

      setPreviewMap(Object.fromEntries(entries));
      setLoading(false);
    })();

    return () => {
      disposed = true;
    };
  }, [assets]);

  return { previewMap, loading };
}
