import React, { useEffect } from 'react';
import { AlertTriangle } from 'lucide-react';
import { useProgressionStore } from './store/useProgressionStore';
import { TopHudBar } from './components/layout/TopHudBar';
import { SplitKioskView } from './components/layout/SplitKioskView';
import { SkillTreeGraph } from './components/map/SkillTreeGraph';
import { WalletExporter } from './components/academic/WalletExporter';
import { ApiKeyModal } from './components/settings/ApiKeyModal';
import { CalibrationOnboarding } from './components/kiosk/CalibrationOnboarding';

export const App: React.FC = () => {
  const { activeView, loadCurriculum, error, clockTamperAlert } = useProgressionStore();

  useEffect(() => {
    loadCurriculum();
  }, [loadCurriculum]);

  return (
    <div
      className="h-screen w-screen flex flex-col overflow-hidden"
      style={{
        backgroundColor: 'var(--bg-canvas)',
        color: 'var(--text-primary)',
      }}
    >
      <TopHudBar />

      {clockTamperAlert && (
        <div
          role="alert"
          className="px-4 py-2 text-xs flex items-center gap-2 border-b"
          style={{
            backgroundColor: 'rgba(245, 158, 11, 0.14)',
            borderColor: 'var(--status-warning)',
            color: 'var(--status-warning)',
          }}
        >
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span>
            Divergência de relógio detectada em relação à última atividade registrada. Sua ofensiva (streak) foi preservada de forma congelada.
          </span>
        </div>
      )}

      {error && (
        <div
          role="alert"
          className="px-4 py-2 text-xs border-b"
          style={{
            backgroundColor: 'rgba(244, 63, 94, 0.14)',
            borderColor: 'var(--status-danger)',
            color: 'var(--status-danger)',
          }}
        >
          {error}
        </div>
      )}

      {activeView === 'kiosk' && <SplitKioskView />}
      {activeView === 'map' && <SkillTreeGraph />}
      {activeView === 'wallet' && <WalletExporter />}

      <ApiKeyModal />
      <CalibrationOnboarding />
    </div>
  );
};

export default App;
