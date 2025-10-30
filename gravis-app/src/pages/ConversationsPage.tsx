import React from 'react';
import { ConversationsWindow } from '../components/ConversationsWindow';
import { invoke } from '@tauri-apps/api/core';
import type { Conversation } from '@/lib/conversation-manager';

const ConversationsPage: React.FC = () => {
  const handleResumeConversation = async (conversation: Conversation) => {
    try {
      // Émettre un événement vers la fenêtre principale pour reprendre la conversation
      await invoke('broadcast_to_window', {
        windowLabel: 'main',
        event: 'resume_conversation',
        payload: { conversation }
      });
      console.log('📤 Demande de reprise de conversation envoyée:', conversation.title);
    } catch (error) {
      console.error('❌ Erreur lors de l\'envoi de la reprise:', error);
    }
  };

  const handleCopyMessage = (content: string) => {
    console.log('📋 Message copié:', content.substring(0, 50) + '...');
    // Le presse-papiers est déjà géré dans ConversationsWindow
  };

  return (
    <ConversationsWindow 
      onResumeConversation={handleResumeConversation}
      onCopyMessage={handleCopyMessage}
    />
  );
};

export default ConversationsPage;