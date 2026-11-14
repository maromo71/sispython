-- ============================================================================
-- MIGRATION: 0001_initial_schema.sql
-- PROJETO: LogicKiosk (Tauri v2 + Rust + SQLite Local-First)
-- ============================================================================

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- 1. Tabela de Perfis de Estudantes
CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    theme_preference TEXT NOT NULL DEFAULT 'dark_tech' CHECK (theme_preference IN ('dark_tech', 'light_tech')),
    total_xp INTEGER NOT NULL DEFAULT 0,
    current_level_rank INTEGER NOT NULL DEFAULT 1,
    logic_fragments INTEGER NOT NULL DEFAULT 0,
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    last_activity_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Tabela de Módulos Pedagógicos
CREATE TABLE IF NOT EXISTS modules (
    id TEXT PRIMARY KEY NOT NULL,
    order_index INTEGER NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    required_xp_to_unlock INTEGER NOT NULL DEFAULT 0
);

-- 3. Tabela de Fases (Quiosques de Desafios)
CREATE TABLE IF NOT EXISTS levels (
    id TEXT PRIMARY KEY NOT NULL,
    module_id TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    title TEXT NOT NULL,
    level_type TEXT NOT NULL DEFAULT 'STANDARD' CHECK (level_type IN ('STANDARD', 'FIXATION', 'BOSS')),
    problem_statement_md TEXT NOT NULL,
    starter_code TEXT NOT NULL,
    ast_must_contain_json TEXT NOT NULL DEFAULT '[]',
    ast_forbidden_json TEXT NOT NULL DEFAULT '[]',
    base_xp_reward INTEGER NOT NULL DEFAULT 100,
    expected_time_seconds INTEGER NOT NULL DEFAULT 180,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    UNIQUE (module_id, order_index)
);

-- 4. Tabela de Casos de Teste (Públicos e Ocultos de Borda)
CREATE TABLE IF NOT EXISTS test_cases (
    id TEXT PRIMARY KEY NOT NULL,
    level_id TEXT NOT NULL,
    input_data_json TEXT NOT NULL,
    expected_output_json TEXT NOT NULL,
    is_hidden_edge_case INTEGER NOT NULL DEFAULT 0 CHECK (is_hidden_edge_case IN (0, 1)),
    failure_concept_tag TEXT NOT NULL,
    FOREIGN KEY (level_id) REFERENCES levels(id) ON DELETE CASCADE
);

-- 5. Tabela de Progresso e Maestria do Aluno por Fase
CREATE TABLE IF NOT EXISTS user_progress (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    level_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'LOCKED' CHECK (status IN ('LOCKED', 'UNLOCKED', 'COMPLETED_MASTERY')),
    attempts_count INTEGER NOT NULL DEFAULT 0,
    hints_used_count INTEGER NOT NULL DEFAULT 0,
    best_execution_time_ms INTEGER,
    earned_xp INTEGER NOT NULL DEFAULT 0,
    needs_fixation_review INTEGER NOT NULL DEFAULT 0 CHECK (needs_fixation_review IN (0, 1)),
    completed_at TEXT,
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (level_id) REFERENCES levels(id) ON DELETE CASCADE,
    UNIQUE (profile_id, level_id)
);

-- 6. Histórico de Submissões de Código
CREATE TABLE IF NOT EXISTS submissions (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    level_id TEXT NOT NULL,
    submitted_code TEXT NOT NULL,
    verdict TEXT NOT NULL CHECK (verdict IN ('PASSED', 'AST_REJECTED', 'TEST_FAILED', 'TIMEOUT_LOOP', 'SYNTAX_ERROR')),
    execution_time_ms INTEGER NOT NULL DEFAULT 0,
    variable_trace_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (level_id) REFERENCES levels(id) ON DELETE CASCADE
);

-- 7. Registro de Interações com o Agente Mentor Socrático
CREATE TABLE IF NOT EXISTS socratic_interactions (
    id TEXT PRIMARY KEY NOT NULL,
    submission_id TEXT NOT NULL,
    provider_used TEXT NOT NULL CHECK (provider_used IN ('GEMINI_3_8_FLASH', 'GEMINI_3_5_LITE', 'GEMMA_4_26B', 'LOCAL_OLLAMA', 'LOCAL_RUST_ENGINE')),
    diagnostic_category TEXT NOT NULL,
    socratic_question TEXT NOT NULL,
    focus_line INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (submission_id) REFERENCES submissions(id) ON DELETE CASCADE
);

-- 8. Catálogo de Insígnias (Badges)
CREATE TABLE IF NOT EXISTS badges (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    icon_name TEXT NOT NULL,
    rule_condition_json TEXT NOT NULL
);

-- 9. Insígnias Conquistadas pelo Aluno
CREATE TABLE IF NOT EXISTS user_badges (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    badge_id TEXT NOT NULL,
    unlocked_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (badge_id) REFERENCES badges(id) ON DELETE CASCADE,
    UNIQUE (profile_id, badge_id)
);

-- 10. Carteiras de Progresso Assinadas (Verificação Acadêmica SHA-256)
CREATE TABLE IF NOT EXISTS signed_certificates (
    id TEXT PRIMARY KEY NOT NULL,
    profile_id TEXT NOT NULL,
    module_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    sha256_signature TEXT NOT NULL,
    issued_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE
);

-- Índices para Performance de Consultas Locais
CREATE INDEX IF NOT EXISTS idx_levels_module ON levels(module_id, order_index);
CREATE INDEX IF NOT EXISTS idx_test_cases_level ON test_cases(level_id);
CREATE INDEX IF NOT EXISTS idx_user_progress_lookup ON user_progress(profile_id, level_id);
CREATE INDEX IF NOT EXISTS idx_submissions_profile_level ON submissions(profile_id, level_id);

-- ============================================================================
-- SEED INICIAL: PERFIL PADRÃO, INSÍGNIAS E MÓDULO 1 COMPLETO (5 FASES)
-- ============================================================================

INSERT OR IGNORE INTO profiles (id, display_name, theme_preference, total_xp, current_level_rank, logic_fragments, current_streak_days)
VALUES ('prof_local_01', 'Estudante de Lógica', 'dark_tech', 0, 1, 0, 1);

INSERT OR IGNORE INTO modules (id, order_index, title, description, required_xp_to_unlock)
VALUES (
    'mod_01_fundamentos',
    1,
    'Módulo 1: Fundamentos de Lógica e Controle de Fluxo',
    'Construa a base do pensamento computacional dominando variáveis, decisões condicionais, laços de repetição com sentinela e filtragem de vetores.',
    0
);

-- Fase 1.1: Variáveis e Expressões Aritméticas
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_01_variaveis', 'mod_01_fundamentos', 1,
    '1.1 Calibração de Energia: Variáveis e Operadores', 'STANDARD',
    '### Missão: Eficiência do Inversor Solar
Crie a função `calcular_energia_liquida(geracao_kwh, consumo_kwh)` que receba a energia gerada e a energia consumida, e retorne o saldo líquido multiplicado pelo fator de eficiência `0.95` (arredondado em 2 casas decimais com `round(valor, 2)`).',
    'def calcular_energia_liquida(geracao_kwh, consumo_kwh):
    # Calcule o saldo e aplique o fator de eficiência 0.95
    saldo = 0
    return saldo
',
    '["FunctionDef", "Assign", "BinOp", "Return"]',
    '["Import", "Eval", "Exec", "While", "For"]',
    100, 120
);

-- Fase 1.2: Estruturas Condicionais
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_02_condicionais', 'mod_01_fundamentos', 2,
    '1.2 Guardião de Tensão: Decisões Condicionais', 'STANDARD',
    '### Missão: Classificador de Estabilidade
Implemente `classificar_tensao(volts)` que retorne:
- `"SUBTENSAO"` se `volts` for menor que 195;
- `"SOBRETENSAO"` se `volts` for maior que 240;
- `"NORMAL"` caso esteja dentro da faixa segura (inclusive nos limites 195 e 240).',
    'def classificar_tensao(volts):
    # Utilize estruturas condicionais (if / elif / else)
    pass
',
    '["FunctionDef", "If", "Compare", "Return"]',
    '["Import", "Eval", "Exec"]',
    150, 180
);

-- Fase 1.3: Laço de Repetição While
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_03_laco_while', 'mod_01_fundamentos', 3,
    '1.3 Acumulador de Carga: O Laço While', 'STANDARD',
    '### Missão: Carga Progressiva de Bateria
Implemente `ciclos_ate_meta(carga_inicial, incremento_por_ciclo, meta_maxima)` usando obrigatoriamente um laço `while`.
Some `incremento_por_ciclo` à carga atual a cada ciclo enquanto a carga for menor que `meta_maxima` e retorne a quantidade total de ciclos realizados.',
    'def ciclos_ate_meta(carga_inicial, incremento_por_ciclo, meta_maxima):
    ciclos = 0
    carga_atual = carga_inicial
    # Implemente o laço while abaixo
    
    return ciclos
',
    '["FunctionDef", "While", "Compare", "Return"]',
    '["For", "Import", "Eval", "Exec"]',
    200, 240
);

-- Fase 1.4: Vetores e Laço For
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_04_vetores', 'mod_01_fundamentos', 4,
    '1.4 Filtro de Sensores: Vetores e Iteração', 'STANDARD',
    '### Missão: Limpeza de Ruído em Telemetria
Implemente `filtrar_leituras_validas(leituras)` que receba uma lista de números e retorne uma nova lista contendo apenas os valores maiores que zero, percorrendo os elementos com um laço `for`.',
    'def filtrar_leituras_validas(leituras):
    validas = []
    # Percorra a lista leituras com for e adicione apenas valores positivos
    
    return validas
',
    '["FunctionDef", "For", "If", "Return"]',
    '["Import", "Eval", "Exec"]',
    250, 240
);

-- Fase 1.5: BOSS LEVEL do Módulo 1
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_05_boss_modulo1', 'mod_01_fundamentos', 5,
    '1.5 DESAFIO CHEFÃO: Auditoria de Estação Autônoma', 'BOSS',
    '### Missão Chefão (Sem Pistas Socráticas!)
Implemente `auditar_estacao(leituras_kw, limite_alerta)`.
Você deve percorrer a lista `leituras_kw` ignorando valores negativos (erros de sensor) e interromper a leitura imediatamente (`break`) caso encontre o valor sentinela `9999`.
Retorne um dicionário no formato `{"total_valido": soma, "alertas": qtd_acima_do_limite}` onde `alertas` conta quantas leituras válidas foram estritamente maiores que `limite_alerta`.',
    'def auditar_estacao(leituras_kw, limite_alerta):
    soma = 0
    alertas = 0
    # Demonstre sua maestria combinando laços, condicionais e parada por sentinela
    
    return {"total_valido": soma, "alertas": alertas}
',
    '["FunctionDef", "For", "If", "Break", "Return"]',
    '["Import", "Eval", "Exec"]',
    500, 360
);

-- Casos de Teste (Públicos e Ocultos)
INSERT OR IGNORE INTO test_cases (id, level_id, input_data_json, expected_output_json, is_hidden_edge_case, failure_concept_tag) VALUES
-- Testes Nível 1.1
('tc_1_1_pub1', 'lvl_01_variaveis', '[100, 20]', '76.0', 0, 'ARITHMETIC_FORMULA'),
('tc_1_1_pub2', 'lvl_01_variaveis', '[50, 50]', '0.0', 0, 'ZERO_BALANCE'),
('tc_1_1_hid1', 'lvl_01_variaveis', '[10, 30]', '-19.0', 1, 'NEGATIVE_NET_BALANCE'),
-- Testes Nível 1.2
('tc_1_2_pub1', 'lvl_02_condicionais', '[220]', '"NORMAL"', 0, 'NOMINAL_RANGE'),
('tc_1_2_pub2', 'lvl_02_condicionais', '[180]', '"SUBTENSAO"', 0, 'LOW_BOUNDARY'),
('tc_1_2_hid1', 'lvl_02_condicionais', '[195]', '"NORMAL"', 1, 'EXACT_BOUNDARY_INCLUSION_195'),
('tc_1_2_hid2', 'lvl_02_condicionais', '[240]', '"NORMAL"', 1, 'EXACT_BOUNDARY_INCLUSION_240'),
('tc_1_2_hid3', 'lvl_02_condicionais', '[241]', '"SOBRETENSAO"', 1, 'HIGH_BOUNDARY'),
-- Testes Nível 1.3
('tc_1_3_pub1', 'lvl_03_laco_while', '[20, 15, 80]', '4', 0, 'WHILE_INCREMENT_COUNT'),
('tc_1_3_pub2', 'lvl_03_laco_while', '[10, 25, 50]', '2', 0, 'NON_EXACT_MULTIPLE_TARGET'),
('tc_1_3_hid1', 'lvl_03_laco_while', '[100, 10, 80]', '0', 1, 'ALREADY_AT_OR_ABOVE_TARGET'),
-- Testes Nível 1.4
('tc_1_4_pub1', 'lvl_04_vetores', '[[10, -5, 20, 0, 15]]', '[10, 20, 15]', 0, 'FILTER_POSITIVE_ONLY'),
('tc_1_4_hid1', 'lvl_04_vetores', '[[-1, -2, 0]]', '[]', 1, 'ALL_INVALID_RETURNS_EMPTY_LIST'),
-- Testes Nível 1.5 (Boss)
('tc_1_5_pub1', 'lvl_05_boss_modulo1', '[[10, -3, 25, 30, 9999, 50], 20]', '{"total_valido": 65, "alertas": 2}', 0, 'SENTINEL_BREAK_AND_ACCUMULATION'),
('tc_1_5_hid1', 'lvl_05_boss_modulo1', '[[9999, 100, 200], 10]', '{"total_valido": 0, "alertas": 0}', 1, 'IMMEDIATE_SENTINEL_AT_INDEX_ZERO');

-- Progresso Inicial do Aluno (Apenas Fase 1.1 Desbloqueada - Hard Gatekeeping)
INSERT OR IGNORE INTO user_progress (id, profile_id, level_id, status) VALUES
('up_01', 'prof_local_01', 'lvl_01_variaveis', 'UNLOCKED'),
('up_02', 'prof_local_01', 'lvl_02_condicionais', 'LOCKED'),
('up_03', 'prof_local_01', 'lvl_03_laco_while', 'LOCKED'),
('up_04', 'prof_local_01', 'lvl_04_vetores', 'LOCKED'),
('up_05', 'prof_local_01', 'lvl_05_boss_modulo1', 'LOCKED');

-- Catálogo de Insígnias
INSERT OR IGNORE INTO badges (id, code, title, description, icon_name, rule_condition_json) VALUES
('bdg_01', 'FIRST_CIRCUIT', 'Primeiro Circuito Ativo', 'Aprovou o primeiro quiosque de código na 1ª tentativa.', 'zap', '{"type": "FIRST_TRY_PASS"}'),
('bdg_02', 'AUTONOMOUS_MIND', 'Mente Autônoma', 'Concluiu 3 fases seguidas sem solicitar pistas socráticas.', 'brain', '{"type": "ZERO_HINTS_STREAK", "count": 3}'),
('bdg_03', 'BOSS_SLAYER_M1', 'Mestre dos Fundamentos', 'Venceu o Desafio Chefão do Módulo 1 sem auxílio.', 'shield-check', '{"type": "BOSS_CLEARED", "module_id": "mod_01_fundamentos"}');
