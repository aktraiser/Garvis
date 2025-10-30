import "./App.css";
import { CommandInterface } from "./components/CommandInterface";
import { RagPage } from "./pages/RagPage";
import { SettingsPage } from "./pages/SettingsPage";
import { ModelSelectorPage } from "./pages/ModelSelectorPage";
import ConversationsPage from "./pages/ConversationsPage";

function App() {
  // Simple routing based on URL hash and pathname
  const pathname = window.location.pathname;
  const hash = window.location.hash;
  
  if (pathname === '/rag' || hash === '#rag') {
    return <RagPage />;
  }
  
  if (pathname === '/settings' || hash === '#settings') {
    return <SettingsPage />;
  }
  
  if (pathname === '/model_selector' || hash === '#model_selector') {
    return <ModelSelectorPage />;
  }
  
  if (pathname === '/conversations' || hash === '#conversations') {
    return <ConversationsPage />;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <main className="max-w-7xl mx-auto">
        <CommandInterface />
      </main>
    </div>
  );
}

export default App;
