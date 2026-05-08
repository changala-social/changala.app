import React, { Suspense } from "react";
import { Routes, Route } from "react-router-dom";
import { Layout } from "./components/layout/Layout";
import { ProtectedRoute } from "./components/layout/ProtectedRoute";
import { LoadingSpinner } from "./components/common/LoadingSpinner";

// -- Public pages --
const LandingPage = React.lazy(() => import("./pages/LandingPage"));
const CourseCatalog = React.lazy(() => import("./pages/CourseCatalog"));
const CourseDetail = React.lazy(() => import("./pages/CourseDetail"));
const SessionDetail = React.lazy(() => import("./pages/SessionDetail"));
const NoteViewer = React.lazy(() => import("./pages/NoteViewer"));
const BrainFeed = React.lazy(() => import("./pages/BrainFeed"));
const BrainNodeDetail = React.lazy(() => import("./pages/BrainNodeDetail"));
const GraphExplorer = React.lazy(() => import("./pages/GraphExplorer"));
const ArchiveBrowser = React.lazy(() => import("./pages/ArchiveBrowser"));
const SearchPage = React.lazy(() => import("./pages/SearchPage"));
const ProfilePage = React.lazy(() => import("./pages/ProfilePage"));
const LoginPage = React.lazy(() => import("./pages/LoginPage"));

// -- Protected pages --
const Dashboard = React.lazy(() => import("./pages/Dashboard"));
const LiveSession = React.lazy(() => import("./pages/LiveSession"));
const NoteEditor = React.lazy(() => import("./pages/NoteEditor"));
const BrainNodeEditor = React.lazy(() => import("./pages/BrainNodeEditor"));
const AdminPanel = React.lazy(() => import("./pages/AdminPanel"));

const fallback = <LoadingSpinner />;

function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        {/* Public routes */}
        <Route
          path="/"
          element={
            <Suspense fallback={fallback}>
              <LandingPage />
            </Suspense>
          }
        />
        <Route
          path="/courses"
          element={
            <Suspense fallback={fallback}>
              <CourseCatalog />
            </Suspense>
          }
        />
        <Route
          path="/course/:uri"
          element={
            <Suspense fallback={fallback}>
              <CourseDetail />
            </Suspense>
          }
        />
        <Route
          path="/session/:uri"
          element={
            <Suspense fallback={fallback}>
              <SessionDetail />
            </Suspense>
          }
        />
        <Route
          path="/note/:cid"
          element={
            <Suspense fallback={fallback}>
              <NoteViewer />
            </Suspense>
          }
        />
        <Route
          path="/brain"
          element={
            <Suspense fallback={fallback}>
              <BrainFeed />
            </Suspense>
          }
        />
        <Route
          path="/brain/:uri"
          element={
            <Suspense fallback={fallback}>
              <BrainNodeDetail />
            </Suspense>
          }
        />
        <Route
          path="/graph/:uri"
          element={
            <Suspense fallback={fallback}>
              <GraphExplorer />
            </Suspense>
          }
        />
        <Route
          path="/archive"
          element={
            <Suspense fallback={fallback}>
              <ArchiveBrowser />
            </Suspense>
          }
        />
        <Route
          path="/search"
          element={
            <Suspense fallback={fallback}>
              <SearchPage />
            </Suspense>
          }
        />
        <Route
          path="/profile/:did"
          element={
            <Suspense fallback={fallback}>
              <ProfilePage />
            </Suspense>
          }
        />
        <Route
          path="/login"
          element={
            <Suspense fallback={fallback}>
              <LoginPage />
            </Suspense>
          }
        />

        {/* Protected routes */}
        <Route
          path="/dashboard"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <Dashboard />
              </Suspense>
            </ProtectedRoute>
          }
        />
        <Route
          path="/session/:uri/live"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <LiveSession />
              </Suspense>
            </ProtectedRoute>
          }
        />
        <Route
          path="/note/new"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <NoteEditor />
              </Suspense>
            </ProtectedRoute>
          }
        />
        <Route
          path="/brain/new"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <BrainNodeEditor />
              </Suspense>
            </ProtectedRoute>
          }
        />
        <Route
          path="/brain/:uri/edit"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <BrainNodeEditor />
              </Suspense>
            </ProtectedRoute>
          }
        />
        <Route
          path="/admin"
          element={
            <ProtectedRoute>
              <Suspense fallback={fallback}>
                <AdminPanel />
              </Suspense>
            </ProtectedRoute>
          }
        />
      </Route>
    </Routes>
  );
}

export default App;
