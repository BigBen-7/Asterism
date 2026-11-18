import type { Metadata } from "next";
import type { ReactNode } from "react";
import "./globals.css";

export const metadata: Metadata = {
  title: "Asterism Credentials",
  description: "Issue, revoke and verify credentials on Stellar testnet.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <main>
          <nav className="nav">
            <a href="/">Issuer dashboard</a>
            <a href="/verify">Verify a credential</a>
          </nav>
          {children}
          <p className="muted">Testnet only. Unaudited software; do not use with real value.</p>
        </main>
      </body>
    </html>
  );
}
