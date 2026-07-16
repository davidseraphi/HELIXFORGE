import type { Metadata } from "next";
import { Shell } from "@helixforge/ui";
import "./globals.css";

export const metadata: Metadata = {
  title: "HelixForge Console",
  description: "Sovereign control plane for the HelixForge ecosystem",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Shell>{children}</Shell>
      </body>
    </html>
  );
}
