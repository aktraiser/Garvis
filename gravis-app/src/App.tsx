import "./App.css";
import { CommandInterface } from "./components/CommandInterface";
import { RagPage } from "./pages/RagPage";
import { SettingsPage } from "./pages/SettingsPage";
import { ModelSelectorPage } from "./pages/ModelSelectorPage";
import ConversationsPage from "./pages/ConversationsPage";
import { DirectChatPage } from "./pages/DirectChatPage";
import { OCRViewerPage } from "./pages/OCRViewerPage";

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

  if (pathname === '/direct-chat' || hash === '#direct-chat') {
    return <DirectChatPage />;
  }

  if (hash.startsWith('#ocr-viewer')) {
    return <OCRViewerPage />;
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
