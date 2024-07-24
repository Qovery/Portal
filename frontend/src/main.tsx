import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider, createBrowserRouter } from "react-router-dom";
import "./main.css";

import SelfService from "./pages/self-service/self-service";
import { Providers } from "./providers";
import App from "./pages/app";
import { RunHistory } from "./pages/self-service/run-history/run-history";
import CatalogList from "./pages/self-service/catalog-list/catalog-list";
import { Details } from "./pages/self-service/details/details";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <div>Oops! There was an error.</div>,
    children: [
      {
        path: "/self-service",
        element: <SelfService />,
        children: [
          {
            element: <CatalogList />,
            index: true,
          },
          {
            path: "/self-service/run-history",
            element: <RunHistory />,
          },
          {
            path: "/self-service/:taskId/details",
            element: <Details />,
          },
        ],
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Providers>
      <RouterProvider router={router} />
    </Providers>
  </React.StrictMode>,
);
