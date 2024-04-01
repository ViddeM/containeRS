export const dynamic = "force-dynamic";
import { RepositoryView } from "@/components/views/repository-view/RepositoryView";

export default async function Page({
  params,
}: {
  params: { repository: string };
}) {
  return (
    <main className="main">
      <RepositoryView repositoryName={params.repository} />
    </main>
  );
}
