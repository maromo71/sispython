import { invoke } from '@tauri-apps/api/core';
import type {
  CurriculumPayload,
  ProfileRecord,
  SignedWalletCertificate,
  SignedWalletExport,
  SocraticHintResponse,
  SubmitCodeResponse,
} from '../types/ipc';

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export const ipcClient = {
  getCurriculum(): Promise<CurriculumPayload> {
    return invoke<CurriculumPayload>('get_curriculum');
  },

  submitCode(
    levelId: string,
    sourceCode: string,
    elapsedSeconds?: number
  ): Promise<SubmitCodeResponse> {
    return invoke<SubmitCodeResponse>('submit_code', {
      levelId,
      sourceCode,
      elapsedSeconds: elapsedSeconds ?? null,
    });
  },

  requestSocraticHint(
    levelId: string,
    sourceCode: string,
    failedConceptTag?: string | null
  ): Promise<SocraticHintResponse> {
    return invoke<SocraticHintResponse>('request_socratic_hint', {
      levelId,
      sourceCode,
      failedConceptTag: failedConceptTag ?? null,
    });
  },

  toggleTheme(theme: 'dark_tech' | 'light_tech'): Promise<string> {
    return invoke<string>('toggle_theme', { theme });
  },

  updateStudentName(displayName: string): Promise<ProfileRecord> {
    return invoke<ProfileRecord>('update_student_name', { displayName });
  },

  saveApiKey(apiKey: string): Promise<boolean> {
    return invoke<boolean>('save_api_key', { apiKey });
  },

  getApiKeyStatus(): Promise<boolean> {
    return invoke<boolean>('get_api_key_status');
  },

  exportSignedWallet(moduleId: string): Promise<SignedWalletExport> {
    return invoke<SignedWalletExport>('export_signed_wallet', { moduleId });
  },

  verifySignedWallet(certificate: SignedWalletCertificate): Promise<boolean> {
    return invoke<boolean>('verify_signed_wallet', { certificate });
  },
};
