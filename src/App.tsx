import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Toaster } from "sonner";
import { Sidebar, SidebarSection, SECTIONS_CONFIG } from "./components/Sidebar";
import Footer from "./components/footer/Footer";
import "./App.css";

function App() {
  const [currentSection, setCurrentSection] =
    useState<SidebarSection>("redactor");

  useEffect(() => {
    const unlisten = listen("show-redactor", () => {
      setCurrentSection("redactor");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="h-screen flex flex-col select-none cursor-default">
      <Toaster
        theme="system"
        toastOptions={{
          unstyled: true,
          classNames: {
            toast:
              "bg-background border border-mid-gray/20 rounded-lg shadow-lg px-4 py-3 flex items-center gap-3 text-sm",
            title: "font-medium",
            description: "text-mid-gray",
          },
        }}
      />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          activeSection={currentSection}
          onSectionChange={setCurrentSection}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="flex flex-col items-center p-4 gap-4">
              {(() => {
                const config = SECTIONS_CONFIG[currentSection];
                if (!config?.enabled()) return null;
                const Component = config.component;
                return (
                  <div className="w-full">
                    <Component />
                  </div>
                );
              })()}
            </div>
          </div>
        </div>
      </div>
      <Footer />
    </div>
  );
}

export default App;
