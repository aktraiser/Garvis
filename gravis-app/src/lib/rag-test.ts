// Test des commandes RAG Phase 1
import { invoke } from '@tauri-apps/api/core';

export async function testRagCommands() {
  try {
    console.log('🧪 Testing RAG Commands Phase 1...');
    
    // Test statut RAG
    const status = await invoke('rag_get_status') as string;
    console.log('✅ RAG Status:', status);
    
    // Test création groupe
    const group = await invoke('rag_create_group', { name: 'Test Group' });
    console.log('✅ Created Group:', group);
    
    // Test liste groupes
    const groups = await invoke('rag_list_groups');
    console.log('✅ Listed Groups:', groups);
    
    console.log('🎉 All RAG commands working!');
    return { success: true, status, group, groups };
    
  } catch (error) {
    console.error('❌ RAG Commands Error:', error);
    return { success: false, error };
  }
}