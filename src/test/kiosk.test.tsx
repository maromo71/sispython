import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { useThemeStore } from '../store/useThemeStore';
import { useKioskStore } from '../store/useKioskStore';
import { isCodeEmptyOrCommentsOnly, useProgressionStore } from '../store/useProgressionStore';
import { TopHudBar } from '../components/layout/TopHudBar';
import { ExecutionConsole } from '../components/kiosk/ExecutionConsole';
import { SkillTreeGraph } from '../components/map/SkillTreeGraph';
import { MissionPanel } from '../components/kiosk/MissionPanel';
import type { CurriculumPayload, LevelProgressRecord, SubmitCodeResponse } from '../types/ipc';

const sampleLevel: LevelProgressRecord = {
  id: 'lvl_01_variaveis',
  module_id: 'mod_01_fundamentos',
  order_index: 1,
  title: '1.1 Calibração de Energia',
  level_type: 'STANDARD',
  problem_statement_md: '### Missão\nImplemente `calcular(a, b)`.',
  starter_code: 'def calcular(a, b):\n    return a - b\n',
  must_contain_nodes: ['FunctionDef', 'Return'],
  forbidden_nodes: ['Import'],
  base_xp_reward: 100,
  expected_time_seconds: 120,
  status: 'UNLOCKED',
  attempts_count: 0,
  hints_used_count: 0,
  best_execution_time_ms: null,
  earned_xp: 0,
  needs_fixation_review: false,
  last_submitted_code: null,
};

const bossLevel: LevelProgressRecord = {
  ...sampleLevel,
  id: 'lvl_05_boss_modulo1',
  order_index: 5,
  title: '1.5 DESAFIO CHEFÃO',
  level_type: 'BOSS',
  status: 'LOCKED',
};

const sampleCurriculum: CurriculumPayload = {
  profile: {
    id: 'prof_local_01',
    display_name: 'Estudante Teste',
    theme_preference: 'dark_tech',
    total_xp: 150,
    current_level_rank: 1,
    logic_fragments: 1,
    current_streak_days: 2,
    last_activity_date: '2026-11-14',
  },
  modules: [
    {
      id: 'mod_01_fundamentos',
      order_index: 1,
      title: 'Módulo 1: Fundamentos',
      description: 'Base lógica',
      required_xp_to_unlock: 0,
    },
  ],
  levels: [sampleLevel, bossLevel],
  public_test_cases: {
    lvl_01_variaveis: [
      {
        id: 'tc1',
        input_data_json: '[10, 5]',
        expected_output_json: '5',
        failure_concept_tag: 'ARITHMETIC',
      },
    ],
  },
  badges: [],
  ai_key_configured: false,
  ai_status_label: 'Offline: Motor Nativo Rust',
};

describe('LogicKiosk UI & Stores', () => {
  beforeEach(() => {
    useThemeStore.getState().setTheme('dark-tech', false);
    useProgressionStore.setState({
      curriculum: sampleCurriculum,
      activeView: 'kiosk',
      gatekeeperNotice: null,
    });
    useKioskStore.setState({
      activeLevelId: 'lvl_01_variaveis',
      calibrationDone: true,
    });
  });

  it('alterna instantaneamente entre dark-tech e light-tech no HTML root', () => {
    render(<TopHudBar />);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark-tech');
    const toggleBtn = screen.getByRole('button', { name: /alternar tema/i });
    fireEvent.click(toggleBtn);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light-tech');
  });

  it('detecta e bloqueia submissões vazias ou apenas com comentários', () => {
    expect(isCodeEmptyOrCommentsOnly('   \n  # apenas comentario\n\t')).toBe(true);
    expect(isCodeEmptyOrCommentsOnly('def f(x):\n    return x')).toBe(false);
  });

  it('renderiza estados Empty, Loading, Timeout e Success corretamente no ExecutionConsole', () => {
    const { rerender } = render(
      <ExecutionConsole
        level={sampleLevel}
        publicTestCases={sampleCurriculum.public_test_cases.lvl_01_variaveis}
        isRunning={false}
        result={null}
        hasNextLevel={true}
        onAdvanceNextLevel={() => {}}
      />
    );
    expect(screen.getByTestId('console-empty-state')).toBeInTheDocument();

    rerender(
      <ExecutionConsole
        level={sampleLevel}
        publicTestCases={[]}
        isRunning={true}
        result={null}
        hasNextLevel={true}
        onAdvanceNextLevel={() => {}}
      />
    );
    expect(screen.getByTestId('console-loading-state')).toBeInTheDocument();

    const timeoutResult: SubmitCodeResponse = {
      verdict: 'TIMEOUT_LOOP',
      ast_report: {
        is_valid: true,
        detected_nodes: ['FunctionDef', 'While'],
        missing_required_nodes: [],
        triggered_forbidden_nodes: [],
        suspected_hardcode: false,
        syntax_error_detail: null,
      },
      execution_summary: {
        all_passed: false,
        verdict: 'TIMEOUT_LOOP',
        execution_time_ms: 2001,
        stdout: '',
        stdout_truncated: false,
        variable_trace: [{ step: 1, line: 3, variables: { x: '10' } }],
        test_results: [],
        first_failed_concept_tag: 'INFINITE_LOOP',
        runtime_error: 'TimeoutError',
      },
      socratic_hint: null,
      reward: null,
      updated_profile: sampleCurriculum.profile,
      clock_tamper_detected: false,
    };

    rerender(
      <ExecutionConsole
        level={sampleLevel}
        publicTestCases={[]}
        isRunning={false}
        result={timeoutResult}
        hasNextLevel={false}
        onAdvanceNextLevel={() => {}}
      />
    );
    expect(screen.getByTestId('console-timeout-state')).toBeInTheDocument();

    const successResult: SubmitCodeResponse = {
      ...timeoutResult,
      verdict: 'PASSED',
      execution_summary: {
        ...timeoutResult.execution_summary!,
        all_passed: true,
        verdict: 'PASSED',
        execution_time_ms: 12,
      },
      reward: {
        earned_xp: 125,
        new_total_xp: 275,
        updated_streak_days: 2,
        next_level_unlocked_id: 'lvl_02_condicionais',
        scheduled_fixation_review: false,
        newly_unlocked_badges: [],
      },
    };

    rerender(
      <ExecutionConsole
        level={sampleLevel}
        publicTestCases={[]}
        isRunning={false}
        result={successResult}
        hasNextLevel={true}
        onAdvanceNextLevel={() => {}}
      />
    );
    expect(screen.getByTestId('console-success-state')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /avançar para fase seguinte/i })).toBeInTheDocument();
  });

  it('bloqueia pedido de pistas socráticas em fases BOSS', () => {
    render(
      <MissionPanel
        level={bossLevel}
        code="def auditar(): pass"
        messages={[]}
        isRequestingHint={false}
        onRequestHint={() => {}}
      />
    );
    const hintBtn = screen.getByRole('button', { name: /pistas bloqueadas/i });
    expect(hintBtn).toBeDisabled();
  });

  it('bloqueia acesso a fases LOCKED no SkillTreeGraph (Hard Gatekeeping)', () => {
    render(<SkillTreeGraph />);
    const bossBtn = screen.getByText('1.5 DESAFIO CHEFÃO').closest('button')!;
    fireEvent.click(bossBtn);
    expect(screen.getByRole('alert')).toHaveTextContent(/bloqueada/i);
  });
});
