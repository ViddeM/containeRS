export const dynamic = "force-dynamic";
import { RepositoriesList } from "@/components/views/repositories-list/RepositoriesList";
import { Suspense } from "react";

export default async function Home() {
  return (
    <main className="main">
      <Suspense fallback={<p>Loading...</p>}>
        <RepositoriesList />
      </Suspense>
    </main>
  );
}
