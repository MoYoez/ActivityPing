import type { ComponentProps } from "react";

import { AboutView } from "../about/AboutView";

export function AboutPage(props: ComponentProps<typeof AboutView>) {
  return <AboutView {...props} />;
}
