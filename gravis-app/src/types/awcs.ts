// GRAVIS AWCS - Types TypeScript
// Définitions des types côté frontend en sync avec Rust

/// État d'activation AWCS
export enum AWCSActivationState {
  Disabled = 'disabled',
  PermissionsPending = 'permissions_pending',
  PermissionsGranted = 'permissions_granted',
  Active = 'active',
  Error = 'error',
}

/// Permissions système AWCS
export interface AWCSPermissions {
  accessibility: boolean;
  automation: boolean;
  screenRecording: boolean;
}

/// Informations sur la fenêtre active
export interface WindowInfo {
  app: string;
  title: string;
  pid: number;
  bundleId?: string;     // macOS
  windowClass?: string;  // Windows/Linux
}

/// Informations sur le document
export interface DocumentInfo {
  docType: string;
  path?: string;
  url?: string;
}

/// Données de contenu extraites
export interface ContentData {
  selection?: string;
  fulltext?: string;
  metadata?: any;
}

/// Confidence de l'extraction
export interface ExtractionConfidence {
  textCompleteness: number;
  sourceReliability: number;
  extractionMethod: string;
}

/// Drapeaux de sécurité
export interface SecurityFlags {
  piiRedacted: boolean;
  fullTextBlocked: boolean;
  ocrDegraded: boolean;
}

/// Enveloppe de contexte principale
export interface ContextEnvelope {
  source: WindowInfo;
  document?: DocumentInfo;
  content: ContentData;
  confidence: ExtractionConfidence;
  timestamp: string; // ISO string
  securityFlags?: SecurityFlags;
}

/// Types d'intention
export enum IntentionType {
  Summary = 'Summary',
  Search = 'Search',
  Recommendation = 'Recommendation',
  Translation = 'Translation',
  Explanation = 'Explanation',
  General = 'General',
}

/// Classification d'intention
export interface IntentionClassification {
  intentionType: IntentionType;
  confidence: number;
  keywords: string[];
}

/// Stratégie d'exécution
export interface ExecutionStrategy {
  approach: string;
  estimatedDuration: number; // en ms
  requiresWebSearch: boolean;
  requiresLlm: boolean;
}

/// Action suggérée
export interface SuggestedAction {
  actionType: string;
  label: string;
  description: string;
}

/// Résultat d'intention analysée
export interface IntentionResult {
  classification: IntentionClassification;
  relevantContent: string;
  strategy: ExecutionStrategy;
  suggestedActions: SuggestedAction[];
}

/// Résultat de tâche exécutée
export interface TaskResult {
  taskType: string;
  result: string;
  suggestedActions: SuggestedAction[];
  executionTime: number;
  success: boolean;
}

/// Résultat complet AWCS
export interface AWCSResult {
  context: ContextEnvelope;
  intention: IntentionResult;
  result: TaskResult;
  executionTime: number;
}

/// Coordonnées de sélection
export interface SelectionCoordinates {
  x: number;
  y: number;
  width: number;
  height: number;
}

/// Résultat de sélection utilisateur
export interface SelectionResult {
  text: string;
  confidence: number;
  coordinates?: SelectionCoordinates;
  method: string;
}

/// Configuration AWCS
export interface AWCSConfig {
  enabled: boolean;
  globalShortcut: string;
  extractionTimeout: number;
  maxContentLength: number;
  piiRedactionEnabled: boolean;
  allowedApps: string[];
  blockedApps: string[];
  securityMode: SecurityMode;
}

/// Mode de sécurité
export enum SecurityMode {
  Permissive = 'Permissive',
  Balanced = 'Balanced',
  Strict = 'Strict',
}

/// Métriques AWCS
export interface AWCSMetrics {
  extractionsTotal: number;
  extractionSuccessRate: number;
  avgExtractionTime: number;
  methodDistribution: Record<string, number>;
  appCompatibility: Record<string, number>;
  intentionAccuracy: number;
  userSatisfaction?: number;
}

/// Erreurs AWCS
export class AWCSError extends Error {
  constructor(
    message: string,
    public code: AWCSErrorCode,
    public details?: any
  ) {
    super(message);
    this.name = 'AWCSError';
  }
}

export enum AWCSErrorCode {
  WindowDetectionFailed = 'WindowDetectionFailed',
  ExtractionFailed = 'ExtractionFailed',
  UnsupportedApp = 'UnsupportedApp',
  PermissionsInsufficient = 'PermissionsInsufficient',
  ScriptFailed = 'ScriptFailed',
  OCRFailed = 'OCRFailed',
  IntentAnalysisFailed = 'IntentAnalysisFailed',
  TaskExecutionFailed = 'TaskExecutionFailed',
}

/// Configuration par défaut
export const DEFAULT_AWCS_CONFIG: AWCSConfig = {
  enabled: false,
  globalShortcut: navigator.platform.includes('Mac') ? 'Cmd+Shift+G' : 'Ctrl+Shift+G',
  extractionTimeout: 5000,
  maxContentLength: 100000,
  piiRedactionEnabled: true,
  allowedApps: [
    'Safari',
    'Chrome',
    'Microsoft Word',
    'Code',
    'Firefox',
    'Edge',
  ],
  blockedApps: [
    'Keychain Access',
    '1Password',
    'Bitwarden',
  ],
  securityMode: SecurityMode.Balanced,
};

/// Utilitaires de type
export namespace AWCSUtils {
  /// Vérifie si l'état AWCS est actif
  export function isActive(state: AWCSActivationState): boolean {
    return state === AWCSActivationState.Active;
  }
  
  /// Vérifie si les permissions sont suffisantes
  export function hasRequiredPermissions(permissions: AWCSPermissions): boolean {
    return permissions.accessibility && permissions.automation;
  }
  
  /// Formate le temps d'exécution
  export function formatExecutionTime(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
  
  /// Calcule le score de confidence global
  export function calculateOverallConfidence(context: ContextEnvelope): number {
    const { textCompleteness, sourceReliability } = context.confidence;
    return (textCompleteness * 0.7) + (sourceReliability * 0.3);
  }
  
  /// Détermine l'icône pour une méthode d'extraction
  export function getMethodIcon(method: string): string {
    const icons: Record<string, string> = {
      'dom': '🌐',
      'applescript': '🍎',
      'com': '🪟',
      'accessibility': '♿',
      'ocr': '📷',
      'fallback': '🔧',
    };
    return icons[method] || '🔧';
  }
  
  /// Détermine la couleur pour un niveau de confidence
  export function getConfidenceColor(confidence: number): string {
    if (confidence > 0.9) return 'text-green-600';
    if (confidence > 0.7) return 'text-yellow-600';
    return 'text-red-600';
  }
  
  /// Crée des actions rapides contextuelles
  export function getContextualQuickActions(context: ContextEnvelope): Array<{
    query: string;
    icon: string;
    label: string;
  }> {
    const baseActions = [
      { query: 'Résume ce contenu en 5 points', icon: '📝', label: 'Résumé' },
      { query: 'Propose 3 actions à partir de ce contenu', icon: '💡', label: 'Actions' },
    ];
    
    const app = context.source.app.toLowerCase();
    
    // Actions spécifiques aux navigateurs
    if (app.includes('safari') || app.includes('chrome') || app.includes('firefox')) {
      baseActions.push(
        { query: 'Vérifie les informations de cette page', icon: '🔍', label: 'Vérifier' },
        { query: 'Trouve les liens importants', icon: '🔗', label: 'Liens' }
      );
    }
    
    // Actions spécifiques aux éditeurs de texte
    if (app.includes('word') || app.includes('pages') || app.includes('code')) {
      baseActions.push(
        { query: 'Corrige le style et la grammaire', icon: '✏️', label: 'Corriger' },
        { query: 'Génère un plan pour ce document', icon: '📋', label: 'Plan' }
      );
    }
    
    return baseActions;
  }
  
  /// Valide une configuration AWCS
  export function validateConfig(config: Partial<AWCSConfig>): string[] {
    const errors: string[] = [];
    
    if (config.extractionTimeout !== undefined && config.extractionTimeout < 1000) {
      errors.push('Le timeout d\'extraction doit être d\'au moins 1000ms');
    }
    
    if (config.maxContentLength !== undefined && config.maxContentLength < 1000) {
      errors.push('La taille maximale de contenu doit être d\'au moins 1000 caractères');
    }
    
    if (config.globalShortcut && !/^(Cmd|Ctrl)\+/.test(config.globalShortcut)) {
      errors.push('Le raccourci global doit commencer par Cmd+ ou Ctrl+');
    }
    
    return errors;
  }
}

/// Types d'événements AWCS
export interface AWCSEvents {
  'awcs-activated': { context: ContextEnvelope };
  'awcs-query-processed': { query: string; result: TaskResult };
  'awcs-error': { error: AWCSError };
  'awcs-state-changed': { state: AWCSActivationState };
  'awcs-permissions-changed': { permissions: AWCSPermissions };
}