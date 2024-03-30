export const dynamic = "force-dynamic";
import { RepositoryView } from "@/components/views/repository-view/RepositoryView";
import { Suspense } from "react";

export default async function Page({
  params,
}: {
  params: { repository: string };
}) {
  return (
    <main className="main">
      <Suspense fallback={<p>Loading...</p>}>
        <RepositoryView repositoryName={params.repository} />
      </Suspense>
    </main>
  );
}
