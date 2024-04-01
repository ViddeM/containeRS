export const dynamic = "force-dynamic";
import { RepositoriesList } from "@/components/views/repositories-list/RepositoriesList";

export default async function Home() {
  return (
    <main className="main">
      <RepositoriesList />
    </main>
  );
}
