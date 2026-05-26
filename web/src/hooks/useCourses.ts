import { useXrpcQuery, useXrpcMutation } from "./useXrpc";
import type {
  Course,
  ListCoursesResponse,
  SearchCoursesResponse,
  GetEnrollmentsResponse,
  MyEnrollmentsResponse,
} from "../generated/types";

export function useCourses(
  semester?: string,
  department?: string,
  cursor?: string,
) {
  const params: Record<string, string> = {};
  if (semester) params.semester = semester;
  if (department) params.department = department;
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<ListCoursesResponse>(
    "app.changala.ring.listCourses",
    params,
  );
}

export function useCourse(uri: string) {
  return useXrpcQuery<Course>(
    "app.changala.ring.getCourse",
    { uri },
    { enabled: !!uri },
  );
}

export function useSearchCourses(query: string, cursor?: string) {
  const params: Record<string, string> = { q: query };
  if (cursor) params.cursor = cursor;
  return useXrpcQuery<SearchCoursesResponse>(
    "app.changala.globalview.searchCourses",
    params,
    {
      enabled: query.length > 0,
    },
  );
}

export function useEnrollStudent() {
  return useXrpcMutation<
    { courseUri: string; did: string; enrolledAt: string },
    { courseUri: string; targetDid?: string; slot?: string }
  >("app.changala.ring.enrollStudent");
}

export function useMyEnrollments(did: string) {
  return useXrpcQuery<MyEnrollmentsResponse>(
    "app.changala.ring.getMyEnrollments",
    { did },
    { enabled: !!did },
  );
}

export function useCourseEnrollments(courseUri: string) {
  return useXrpcQuery<GetEnrollmentsResponse>(
    "app.changala.ring.getEnrollments",
    { courseUri },
    { enabled: !!courseUri },
  );
}
