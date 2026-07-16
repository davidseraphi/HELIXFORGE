export const metadata = {
  title: "HelixPulse",
  description: "Sovereign distributed memory & cluster data plane",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
