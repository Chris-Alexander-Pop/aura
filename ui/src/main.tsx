import React from "react"
import ReactDOM from "react-dom/client"
import { HashRouter, Routes, Route } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import "./index.css"

import ControlCenter from "./pages/ControlCenter"
import Sidebar from "./pages/Sidebar"
import Dropdown from "./pages/Dropdown"
import Calendar from "./pages/Calendar"
import BarStrip from "./pages/BarStrip"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 10_000,
      refetchOnWindowFocus: false,
    },
  },
})

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/control-center" element={<ControlCenter />} />
          <Route path="/sidebar" element={<Sidebar />} />
          <Route path="/dropdown" element={<Dropdown />} />
          <Route path="/calendar" element={<Calendar />} />
          <Route path="/bar" element={<BarStrip />} />
          {/* Dev landing page */}
          <Route
            path="/"
            element={
              <div className="flex h-full items-center justify-center gap-6 bg-base text-subtext1 text-sm">
                <a href="#/control-center" className="nav-item">Control Center</a>
                <a href="#/sidebar" className="nav-item">Sidebar</a>
                <a href="#/dropdown" className="nav-item">Dropdown</a>
                <a href="#/calendar" className="nav-item">Calendar</a>
                <a href="#/bar" className="nav-item text-teal font-medium">Bar strip</a>
              </div>
            }
          />
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  </React.StrictMode>
)
