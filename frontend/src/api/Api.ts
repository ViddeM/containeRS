import axios from "axios";
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
  return axios
    .get<T>(`${baseUrl}${endpoint}`)
    .then((res) => {
      console.log("GOT RES", res);

      if (!res.data) {
        return {
          isSuccess: false,
          error: "No data in response?",
        };
      }

      return {
        isSuccess: true,
        data: res.data!!,
      };
    })
    .catch((err) => {
      console.error("Failed to send request, res: ", err);
      return {
        isSuccess: false,
        error: err,
      };
    });
}
