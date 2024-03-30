import { Api } from "@/api/Api";
import { RepositoriesList } from "@/components/views/repositories-list/RepositoriesList";
import { Error } from "@/components/views/error/Error";

export default async function Home() {
  const data = await Api.repositories.getAll();

  if (!data.isSuccess) {
    let error = data.error || "unknown error";
    return <Error message={error} />;
  }

  if (!data.data) {
    return <Error message="No data" />;
  }

  return (
    <main className="main">
      <RepositoriesList repositories={data.data!!.repositories} />
    </main>
  );
}
