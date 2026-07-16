"use client";

import type { ReactNode } from "react";

export type ShellProps = {
  children: ReactNode;
  title?: string;
  subtitle?: string;
};

export function Shell({ children, title = "HelixForge", subtitle = "Sovereign ecosystem console" }: ShellProps) {
  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <span className="logo">⬡</span>
          <div>
            <strong>{title}</strong>
            <div className="muted">{subtitle}</div>
          </div>
        </div>
        <nav className="nav">
          <a href="/">Catalog</a>
          <a href="/core">HelixCore</a>
          <a href="/health">Health</a>
        </nav>
      </header>
      <main className="main">{children}</main>
      <footer className="footer">
        HelixForge · self-hostable · zero-trust · hash-chained audit
      </footer>
    </div>
  );
}
