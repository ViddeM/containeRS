import { Repositories, Repository } from "./Repository";

let baseUrl = "/api";

if (typeof window === "undefined") {
  baseUrl = process.env.NEXT_PUBLIC_BASE_URL || "/api";
}

export const Api = {
  repositories: {
    getAll: () => {
      return get<Repositories>("/repositories");
    },
    getOne: (name: string) => {
      return get<Repository>(`/repositories/${name}`);
    },
  },
};

export type Response<T> = {
  data?: T;
  isSuccess: boolean;
  error?: string;
};

async function get<T>(endpoint: string): Promise<Response<T>> {
  const res = await fetch(`${baseUrl}${endpoint}`, { next: { revalidate: 5 } });

  if (!res.ok) {
    console.error("Failed to send request, res: ", res);
    return {
      isSuccess: false,
      error: "Failed to perform request",
    };
  }

  const body = await res.json();

  console.log("Response body", body);

  return {
    isSuccess: true,
    data: body,
  };
}
