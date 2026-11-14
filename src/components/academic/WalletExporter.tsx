import React, { useState } from 'react';
import {
  Award,
  CheckCircle2,
  Download,
  FileCode2,
  ShieldCheck,
  UserCheck,
  XCircle,
} from 'lucide-react';
import { ipcClient } from '../../services/ipcClient';
import { useProgressionStore } from '../../store/useProgressionStore';
import type { SignedWalletCertificate, SignedWalletExport } from '../../types/ipc';

function triggerDownload(filename: string, content: string, mime: string) {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export const WalletExporter: React.FC = () => {
  const { curriculum, updateStudentName } = useProgressionStore();
  const profile = curriculum?.profile;
  const modules = curriculum?.modules ?? [];
  const activeModule = modules[0];

  const [studentName, setStudentName] = useState(profile?.display_name ?? 'Estudante de Lógica');
  const [nameSaved, setNameSaved] = useState(false);
  const [exportData, setExportData] = useState<SignedWalletExport | null>(null);
  const [isExporting, setIsExporting] = useState(false);

  const [verifyInput, setVerifyInput] = useState('');
  const [verifyResult, setVerifyResult] = useState<'valid' | 'invalid' | 'parse_error' | null>(null);

  const handleSaveName = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!studentName.trim()) return;
    await updateStudentName(studentName);
    setNameSaved(true);
    setTimeout(() => setNameSaved(false), 2000);
  };

  const handleExport = async () => {
    if (!activeModule) return;
    setIsExporting(true);
    try {
      const res = await ipcClient.exportSignedWallet(activeModule.id);
      setExportData(res);
      setVerifyInput(JSON.stringify(res.certificate, null, 2));
      setVerifyResult(null);
    } finally {
      setIsExporting(false);
    }
  };

  const handleVerify = async () => {
    try {
      const parsed = JSON.parse(verifyInput) as SignedWalletCertificate;
      const ok = await ipcClient.verifySignedWallet(parsed);
      setVerifyResult(ok ? 'valid' : 'invalid');
    } catch {
      setVerifyResult('parse_error');
    }
  };

  return (
    <div
      className="flex-1 overflow-y-auto p-6 max-w-5xl w-full mx-auto space-y-6"
      aria-label="Carteira Acadêmica Assinada SHA-256"
    >
      {/* Header Card */}
      <div
        className="p-6 rounded-2xl border space-y-4"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div
              className="p-3 rounded-xl"
              style={{
                backgroundColor: 'rgba(168, 85, 247, 0.16)',
                color: 'var(--accent-gamification)',
              }}
            >
              <Award className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-base font-bold">Carteira de Progresso Assinada (SHA-256)</h2>
              <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
                Exporte seu histórico de fases aprovadas com assinatura criptográfica verificável para comprovação acadêmica.
              </p>
            </div>
          </div>

          <button
            type="button"
            disabled={isExporting || !activeModule}
            onClick={handleExport}
            className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold cursor-pointer disabled:opacity-50"
            style={{
              backgroundColor: 'var(--accent-gamification)',
              color: '#FFFFFF',
            }}
          >
            <ShieldCheck className="w-4 h-4" />
            {isExporting ? 'Gerando Assinatura...' : 'Exportar Carteira de Progresso'}
          </button>
        </div>

        {/* Student Name Form */}
        <form onSubmit={handleSaveName} className="flex flex-wrap items-end gap-3 pt-2 border-t" style={{ borderColor: 'var(--border-subtle)' }}>
          <div className="flex-1 min-w-[220px]">
            <label className="block text-[11px] font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>
              Nome do Estudante no Certificado
            </label>
            <input
              type="text"
              value={studentName}
              onChange={(e) => setStudentName(e.target.value)}
              className="w-full px-3 py-2 rounded-lg border text-xs outline-none"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                borderColor: 'var(--border-subtle)',
                color: 'var(--text-primary)',
              }}
            />
          </div>
          <button
            type="submit"
            className="inline-flex items-center gap-1.5 px-3.5 py-2 rounded-lg border text-xs font-semibold cursor-pointer"
            style={{
              backgroundColor: 'var(--bg-surface-elevated)',
              borderColor: 'var(--border-subtle)',
            }}
          >
            <UserCheck className="w-3.5 h-3.5" />
            {nameSaved ? 'Nome Salvo!' : 'Atualizar Nome'}
          </button>
        </form>
      </div>

      {/* Exported Certificate Preview */}
      {exportData && (
        <div
          className="p-6 rounded-2xl border space-y-4"
          style={{
            backgroundColor: 'var(--bg-surface-panel)',
            borderColor: 'var(--status-success)',
          }}
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <span className="text-[11px] font-bold uppercase tracking-wide" style={{ color: 'var(--status-success)' }}>
                Certificado Assinado Emitido
              </span>
              <h3 className="text-sm font-bold mt-0.5">{exportData.module_title}</h3>
            </div>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() =>
                  triggerDownload(
                    `carteira-logickiosk-${exportData.certificate.profile_id}.json`,
                    JSON.stringify(exportData, null, 2),
                    'application/json'
                  )
                }
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer"
                style={{
                  backgroundColor: 'var(--accent-primary)',
                  color: '#0B0F19',
                }}
              >
                <Download className="w-3.5 h-3.5" />
                Baixar JSON
              </button>
              <button
                type="button"
                onClick={() =>
                  triggerDownload(
                    `carteira-logickiosk-${exportData.certificate.profile_id}.html`,
                    exportData.html_report,
                    'text/html'
                  )
                }
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-xs font-semibold cursor-pointer"
                style={{
                  backgroundColor: 'var(--bg-surface-elevated)',
                  borderColor: 'var(--border-subtle)',
                }}
              >
                <FileCode2 className="w-3.5 h-3.5" />
                Baixar HTML
              </button>
            </div>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
            <div className="p-3 rounded-xl border" style={{ backgroundColor: 'var(--bg-canvas)', borderColor: 'var(--border-subtle)' }}>
              <div style={{ color: 'var(--text-secondary)' }}>Fases Concluídas</div>
              <div className="text-base font-bold font-mono-code mt-0.5">
                {exportData.certificate.completed_levels_count}
              </div>
            </div>
            <div className="p-3 rounded-xl border" style={{ backgroundColor: 'var(--bg-canvas)', borderColor: 'var(--border-subtle)' }}>
              <div style={{ color: 'var(--text-secondary)' }}>XP do Módulo</div>
              <div className="text-base font-bold font-mono-code mt-0.5">
                {exportData.certificate.total_earned_xp} XP
              </div>
            </div>
            <div className="p-3 rounded-xl border" style={{ backgroundColor: 'var(--bg-canvas)', borderColor: 'var(--border-subtle)' }}>
              <div style={{ color: 'var(--text-secondary)' }}>Total Tentativas</div>
              <div className="text-base font-bold font-mono-code mt-0.5">
                {exportData.total_attempts}
              </div>
            </div>
            <div className="p-3 rounded-xl border" style={{ backgroundColor: 'var(--bg-canvas)', borderColor: 'var(--border-subtle)' }}>
              <div style={{ color: 'var(--text-secondary)' }}>Pistas Usadas</div>
              <div className="text-base font-bold font-mono-code mt-0.5">
                {exportData.total_hints_used}
              </div>
            </div>
          </div>

          <div className="space-y-1">
            <div className="text-[11px] font-semibold" style={{ color: 'var(--text-secondary)' }}>
              Assinatura Criptográfica SHA-256:
            </div>
            <div
              className="p-3 rounded-xl font-mono-code text-xs break-all border"
              style={{
                backgroundColor: 'var(--bg-canvas)',
                borderColor: 'var(--border-subtle)',
                color: 'var(--accent-primary)',
              }}
            >
              {exportData.certificate.sha256_signature}
            </div>
          </div>

          {exportData.approved_submissions.length > 0 && (
            <div className="space-y-2">
              <div className="text-xs font-bold">Códigos Aprovados no Certificado</div>
              {exportData.approved_submissions.map((sub) => (
                <div
                  key={sub.level_id}
                  className="p-3 rounded-xl border space-y-1.5"
                  style={{
                    backgroundColor: 'var(--bg-canvas)',
                    borderColor: 'var(--border-subtle)',
                  }}
                >
                  <div className="flex items-center justify-between text-xs font-semibold">
                    <span>{sub.level_title}</span>
                    <span className="font-mono-code" style={{ color: 'var(--status-success)' }}>
                      +{sub.earned_xp} XP · {sub.best_execution_time_ms ?? 0} ms
                    </span>
                  </div>
                  <pre className="font-mono-code text-[11px] overflow-x-auto p-2 rounded" style={{ backgroundColor: 'var(--bg-surface-elevated)' }}>
                    {sub.approved_code}
                  </pre>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Professor Integrity Verifier */}
      <div
        className="p-6 rounded-2xl border space-y-3"
        style={{
          backgroundColor: 'var(--bg-surface-panel)',
          borderColor: 'var(--border-subtle)',
        }}
      >
        <h3 className="text-sm font-bold">Verificador de Autenticidade (Professor)</h3>
        <p className="text-xs" style={{ color: 'var(--text-secondary)' }}>
          Cole o bloco <code className="font-mono-code">certificate</code> JSON para validar se a assinatura SHA-256 é íntegra e não foi adulterada.
        </p>
        <textarea
          rows={6}
          value={verifyInput}
          onChange={(e) => {
            setVerifyInput(e.target.value);
            setVerifyResult(null);
          }}
          placeholder='{"student_name": "...", "sha256_signature": "..."}'
          className="w-full p-3 rounded-xl border font-mono-code text-xs outline-none"
          style={{
            backgroundColor: 'var(--bg-canvas)',
            borderColor: 'var(--border-subtle)',
            color: 'var(--text-primary)',
          }}
        />
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={handleVerify}
            className="px-4 py-2 rounded-lg text-xs font-bold cursor-pointer"
            style={{
              backgroundColor: 'var(--accent-primary)',
              color: '#0B0F19',
            }}
          >
            Verificar Assinatura SHA-256
          </button>

          {verifyResult === 'valid' && (
            <span className="inline-flex items-center gap-1.5 text-xs font-bold" style={{ color: 'var(--status-success)' }}>
              <CheckCircle2 className="w-4 h-4" />
              Assinatura Autêntica e Íntegra!
            </span>
          )}
          {verifyResult === 'invalid' && (
            <span className="inline-flex items-center gap-1.5 text-xs font-bold" style={{ color: 'var(--status-danger)' }}>
              <XCircle className="w-4 h-4" />
              Assinatura Inválida! Os dados foram adulterados.
            </span>
          )}
          {verifyResult === 'parse_error' && (
            <span className="text-xs" style={{ color: 'var(--status-warning)' }}>
              JSON inválido. Cole o objeto certificate completo.
            </span>
          )}
        </div>
      </div>
    </div>
  );
};
