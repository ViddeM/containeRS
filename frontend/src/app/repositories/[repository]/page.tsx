import { RepositoryView } from "@/components/views/repository-view/RepositoryView";

export const Page = ({ params }: { params: { repository: string } }) => {
  return (
    <main className="main">
      <RepositoryView repository={params.repository} />
    </main>
  );
};

export default Page;
