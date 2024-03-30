import { Api } from "@/api/Api";
import { RepositoryView } from "@/components/views/repository-view/RepositoryView";
import { Error } from "@/components/views/error/Error";

export default async function Page({
  params,
}: {
  params: { repository: string };
}) {
  const data = await Api.repositories.getOne(params.repository);

  if (!data.isSuccess) {
    let message = data.error || "unknown error";
    console.error("Failed to retrieve repository from server, response", data);
    return <Error message={message} />;
  }

  if (!data.data) {
    return <Error message={"Got no data from server"} />;
  }

  return (
    <main className="main">
      <RepositoryView repository={data.data} />
    </main>
  );
}
