import type { Metadata } from "next";
import { PT_Sans } from "next/font/google";
import "../resources/styles/colors.scss";
import "../resources/styles/globals.scss";
import { Header } from "@/components/views/header/Header";

const inter = PT_Sans({ subsets: ["latin"], weight: "400" });

export const metadata: Metadata = {
  title: "Containers",
  description: "Container registry viewer",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <Header />
        {children}
      </body>
    </html>
  );
}
