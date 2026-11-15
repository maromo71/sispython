-- =============================================================================
-- LogicKiosk Curriculum Seed — Módulos 2 e 3 (Estruturas de Dados & Algoritmos)
-- Padrão: 4 fases STANDARD + 1 fase BOSS por módulo (mínimo de 5 fases)
-- =============================================================================

-- Módulo 2: Estruturas de Dados, Dicionários e Coleções
INSERT OR IGNORE INTO modules (id, order_index, title, description, required_xp_to_unlock)
VALUES (
    'mod_02_estruturas',
    2,
    'Módulo 2: Estruturas de Dados, Dicionários e Coleções',
    'Domine fatiamento de listas, mapas de frequência com dicionários, compreensões e matrizes.',
    0
);

-- Fase 2.1: Fatiamento e Somas de Janela
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_06_m2_fatiamento', 'mod_02_estruturas', 1,
    '2.1 Janela de Telemetria (Listas e Fatiamento)', 'STANDARD',
    '### Missão
Implemente a função `resumir_janela(leituras, k)` que recebe uma lista de números `leituras` e um inteiro `k` (`k > 0`).
Retorne um dicionário no formato `{"inicio": soma_inicio, "fim": soma_fim}` onde:
- `soma_inicio` é a soma dos `k` primeiros elementos (`leituras[:k]`);
- `soma_fim` é a soma dos `k` últimos elementos (`leituras[-k:]`).
Se a lista `leituras` estiver vazia, retorne `{"inicio": 0, "fim": 0}`.',
    'def resumir_janela(leituras, k):
    # Use fatiamento de listas (slicing) e verifique lista vazia
    return {"inicio": 0, "fim": 0}
',
    '["FunctionDef", "If", "Subscript", "Return"]',
    '["Import", "Eval", "Exec"]',
    200, 180
);

-- Fase 2.2: Mapa de Frequência com Dicionários
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_07_m2_dicionarios', 'mod_02_estruturas', 2,
    '2.2 Mapa de Frequência (Dicionários)', 'STANDARD',
    '### Missão
Implemente `contar_eventos(logs)` que recebe uma lista de strings `logs` (por exemplo `["ok", " WARN ", "OK"]`).
Normalizando espaços nas pontas (`.strip()`) e convertendo para maiúsculas (`.upper()`), retorne um dicionário com a contagem de ocorrências de cada evento.
Strings que ficarem vazias após `.strip()` devem ser ignoradas.',
    'def contar_eventos(logs):
    contagem = {}
    # Normalize cada evento e acumule no dicionário
    return contagem
',
    '["FunctionDef", "For", "Dict", "If", "Return"]',
    '["Import", "Eval", "Exec"]',
    250, 220
);

-- Fase 2.3: Compreensão de Listas e Filtragem
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_08_m2_compreensao', 'mod_02_estruturas', 3,
    '2.3 Filtro Compacto (Compreensão de Listas)', 'STANDARD',
    '### Missão
Implemente `normalizar_ativos(registros)` onde `registros` é uma lista de dicionários no formato `{"nome": str, "ativo": bool, "nota": int}`.
Usando **compreensão de listas** (`[... for ... if ...]`), retorne uma lista contendo apenas os nomes em maiúsculas (`.upper()`) dos registros que possuem `ativo == True` e `nota >= 70`.',
    'def normalizar_ativos(registros):
    # Resolva usando uma compreensão de listas (ListComp)
    return []
',
    '["FunctionDef", "ListComp", "Return"]',
    '["Import", "Eval", "Exec"]',
    280, 240
);

-- Fase 2.4: Matrizes / Listas Aninhadas
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_09_m2_matrizes', 'mod_02_estruturas', 4,
    '2.4 Agregação em Grade (Matrizes)', 'STANDARD',
    '### Missão
Implemente `maior_carga_por_setor(matriz)` que recebe uma lista de listas de números `matriz`.
Cada sublista representa as medições de um setor. Retorne uma nova lista contendo o maior valor de cada setor.
Se algum setor estiver vazio (`[]`), registre `0` para esse setor.',
    'def maior_carga_por_setor(matriz):
    maiores = []
    # Percorra cada linha da matriz tratando setores vazios
    return maiores
',
    '["FunctionDef", "For", "If", "List", "Return"]',
    '["Import", "Eval", "Exec"]',
    320, 260
);

-- Fase 2.5: BOSS LEVEL do Módulo 2
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_10_boss_modulo2', 'mod_02_estruturas', 5,
    '2.5 DESAFIO CHEFÃO: Consolidador de Estoque por Categoria', 'BOSS',
    '### Missão Chefão (Sem Pistas Socráticas!)
Implemente `consolidar_estoque(movimentos)` que recebe uma lista de pares `[categoria, delta]`.
Percorra os movimentos em ordem:
1. Normalize `categoria` com `.strip().upper()`. Se ficar vazia, ignore o item.
2. Se a categoria normalizada for `"FIM"`, interrompa imediatamente o laço com `break`.
3. Acumule `delta` para cada categoria em um dicionário.
4. Ao final, retorne um dicionário contendo **apenas** as categorias cujo saldo final seja estritamente maior que `0`.',
    'def consolidar_estoque(movimentos):
    # Combine dicionários, normalização de texto e sentinela de parada
    return {}
',
    '["FunctionDef", "Dict", "For", "If", "Break", "Return"]',
    '["Import", "Eval", "Exec"]',
    650, 380
);

-- Módulo 3: Decomposição Funcional e Algoritmos
INSERT OR IGNORE INTO modules (id, order_index, title, description, required_xp_to_unlock)
VALUES (
    'mod_03_algoritmos',
    3,
    'Módulo 3: Decomposição Funcional e Algoritmos',
    'Pratique validação defensiva, janelas deslizantes, busca binária e ordenação/escalonamento.',
    0
);

-- Fase 3.1: Validação Defensiva de Faixas
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_11_m3_validacao', 'mod_03_algoritmos', 1,
    '3.1 Validação Defensiva de Faixas', 'STANDARD',
    '### Missão
Implemente `calcular_media_segura(valores, minimo, maximo)` para calcular a média aritmética apenas dos elementos que pertencem ao intervalo `[minimo, maximo]` (inclusive), arredondada para 2 casas decimais com `round(..., 2)`.
Caso nenhum valor esteja dentro da faixa válida (evitando divisão por zero), retorne `None`.',
    'def calcular_media_segura(valores, minimo, maximo):
    # Filtre os valores dentro da faixa e proteja contra divisão por zero
    return None
',
    '["FunctionDef", "For", "If", "Compare", "Return"]',
    '["Import", "Eval", "Exec"]',
    300, 240
);

-- Fase 3.2: Janela Deslizante
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_12_m3_janela_deslizante', 'mod_03_algoritmos', 2,
    '3.2 Maior Pico em Janela Deslizante', 'STANDARD',
    '### Missão
Implemente `maior_soma_janela(serie, janela)` que recebe uma lista de números `serie` e um inteiro `janela`.
Retorne a maior soma possível entre quaisquer `janela` elementos consecutivos de `serie`.
Se `janela <= 0` ou `janela > len(serie)`, retorne `0`.',
    'def maior_soma_janela(serie, janela):
    # Calcule a soma de cada bloco contíguo de tamanho janela
    return 0
',
    '["FunctionDef", "For", "If", "BinOp", "Return"]',
    '["Import", "Eval", "Exec"]',
    340, 260
);

-- Fase 3.3: Busca Binária (Exige While, Proíbe For)
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_13_m3_busca_sentinela', 'mod_03_algoritmos', 3,
    '3.3 Busca Binária em Vetor Ordenado', 'STANDARD',
    '### Missão
Implemente `busca_binaria(lista_ordenada, alvo)` utilizando os ponteiros `esq` e `dir` dentro de um laço `while` (uso de `for` proibido nesta missão).
Retorne o índice inteiro em que `alvo` se encontra em `lista_ordenada`, ou `-1` se o elemento não existir.',
    'def busca_binaria(lista_ordenada, alvo):
    esq = 0
    dir = len(lista_ordenada) - 1
    # Implemente a divisão do intervalo com while
    return -1
',
    '["FunctionDef", "While", "If", "Compare", "Return"]',
    '["Import", "Eval", "Exec", "For"]',
    380, 300
);

-- Fase 3.4: Ranking com Desempate Determinístico
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_14_m3_ranking', 'mod_03_algoritmos', 4,
    '3.4 Ranking com Desempate Determinístico', 'STANDARD',
    '### Missão
Implemente `top_equipes(pontuacoes, limite)` onde `pontuacoes` é uma lista de pares `[equipe, pontos]`.
1. Some todos os pontos por nome de equipe usando um dicionário (uma equipe pode aparecer várias vezes).
2. Ordene as equipes da maior pontuação total para a menor; em caso de empate, use ordem alfabética crescente do nome da equipe.
3. Retorne uma lista com os nomes das `limite` primeiras equipes.',
    'def top_equipes(pontuacoes, limite):
    totais = {}
    # Agregue os pontos, ordene com critério de desempate e retorne até limite nomes
    return []
',
    '["FunctionDef", "Dict", "For", "Assign", "Return"]',
    '["Import", "Eval", "Exec"]',
    420, 320
);

-- Fase 3.5: BOSS LEVEL do Módulo 3
INSERT OR IGNORE INTO levels (
    id, module_id, order_index, title, level_type,
    problem_statement_md, starter_code,
    ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds
) VALUES (
    'lvl_15_boss_modulo3', 'mod_03_algoritmos', 5,
    '3.5 DESAFIO CHEFÃO: Escalonador de Tarefas Prioritárias', 'BOSS',
    '### Missão Chefão (Sem Pistas Socráticas!)
Implemente `escalonar_fila(tarefas, capacidade)` onde `tarefas` é uma lista de dicionários `{"id": str, "custo": int, "prio": int}`.
1. Considere apenas tarefas válidas com `custo > 0` e `prio >= 1`.
2. Ordene as tarefas válidas por `prio` decrescente; em empate, menor `custo`; persistindo o empate, `id` alfabético crescente.
3. Percorra a fila ordenada selecionando cada tarefa cuja adição não ultrapasse `capacidade` restante (`custo <= capacidade_restante`).
Retorne `{"selecionadas": lista_de_ids, "custo_total": soma_dos_custos}`.',
    'def escalonar_fila(tarefas, capacidade):
    # Filtre, ordene com os 3 critérios e faça o escalonamento guloso
    return {"selecionadas": [], "custo_total": 0}
',
    '["FunctionDef", "For", "If", "Dict", "Return"]',
    '["Import", "Eval", "Exec"]',
    800, 420
);

-- Casos de Teste (Módulo 2 e Módulo 3)
INSERT OR IGNORE INTO test_cases (id, level_id, input_data_json, expected_output_json, is_hidden_edge_case, failure_concept_tag) VALUES
-- Testes 2.1
('tc_2_1_pub1', 'lvl_06_m2_fatiamento', '[[10, 20, 30, 40, 50], 2]', '{"inicio": 30, "fim": 90}', 0, 'LIST_SLICING_SUM'),
('tc_2_1_pub2', 'lvl_06_m2_fatiamento', '[[5, 10], 5]', '{"inicio": 15, "fim": 15}', 0, 'SLICE_LARGER_THAN_LIST'),
('tc_2_1_hid1', 'lvl_06_m2_fatiamento', '[[], 3]', '{"inicio": 0, "fim": 0}', 1, 'EMPTY_LIST_GUARD'),
-- Testes 2.2
('tc_2_2_pub1', 'lvl_07_m2_dicionarios', '[["ok", " WARN ", "OK", "err"]]', '{"OK": 2, "WARN": 1, "ERR": 1}', 0, 'DICT_FREQUENCY_NORMALIZATION'),
('tc_2_2_hid1', 'lvl_07_m2_dicionarios', '[["  ", "", "warn", "WARN"]]', '{"WARN": 2}', 1, 'IGNORE_EMPTY_NORMALIZED_STRINGS'),
-- Testes 2.3
('tc_2_3_pub1', 'lvl_08_m2_compreensao', '[[{"nome": "ana", "ativo": true, "nota": 85}, {"nome": "bruno", "ativo": false, "nota": 95}, {"nome": "carla", "ativo": true, "nota": 70}]]', '["ANA", "CARLA"]', 0, 'LISTCOMP_FILTER_AND_UPPER'),
('tc_2_3_hid1', 'lvl_08_m2_compreensao', '[[{"nome": "davi", "ativo": true, "nota": 69}]]', '[]', 1, 'BOUNDARY_GRADE_70'),
-- Testes 2.4
('tc_2_4_pub1', 'lvl_09_m2_matrizes', '[[[12, 45, 30], [], [8, 99, 14]]]', '[45, 0, 99]', 0, 'MATRIX_ROW_MAX_AND_EMPTY_ROW'),
('tc_2_4_hid1', 'lvl_09_m2_matrizes', '[[[-5, -2], [0]]]', '[-2, 0]', 1, 'MATRIX_NEGATIVE_NUMBERS'),
-- Testes 2.5 (Boss M2)
('tc_2_5_pub1', 'lvl_10_boss_modulo2', '[[["cabo", 10], ["sensor", 4], ["cabo", -4], ["sensor", -4], ["FIM", 99], ["cabo", 50]]]', '{"CABO": 6}', 0, 'STOCK_BREAK_AND_POSITIVE_FILTER'),
('tc_2_5_hid1', 'lvl_10_boss_modulo2', '[[["  ", 5], ["fim", 10]]]', '{}', 1, 'EMPTY_KEY_AND_LOWERCASE_FIM'),
-- Testes 3.1
('tc_3_1_pub1', 'lvl_11_m3_validacao', '[[10, 50, 20, -5, 30], 0, 40]', '20.0', 0, 'SAFE_AVERAGE_INCLUSIVE_BOUNDS'),
('tc_3_1_hid1', 'lvl_11_m3_validacao', '[[100, 200], 0, 50]', 'null', 1, 'NO_VALID_ELEMENTS_RETURNS_NONE'),
('tc_3_1_hid2', 'lvl_11_m3_validacao', '[[0, 40], 0, 40]', '20.0', 1, 'EXACT_INCLUSIVE_RANGE'),
-- Testes 3.2
('tc_3_2_pub1', 'lvl_12_m3_janela_deslizante', '[[2, 1, 5, 1, 3, 2], 3]', '9', 0, 'SLIDING_WINDOW_MAX_SUM'),
('tc_3_2_hid1', 'lvl_12_m3_janela_deslizante', '[[4, 2], 5]', '0', 1, 'WINDOW_EXCEEDS_LEN'),
('tc_3_2_hid2', 'lvl_12_m3_janela_deslizante', '[[-5, -2, -1, -8], 2]', '-3', 1, 'NEGATIVE_WINDOW_SUMS'),
-- Testes 3.3
('tc_3_3_pub1', 'lvl_13_m3_busca_sentinela', '[[10, 20, 30, 40, 50], 40]', '3', 0, 'BINARY_SEARCH_FOUND'),
('tc_3_3_hid1', 'lvl_13_m3_busca_sentinela', '[[10, 20, 30], 25]', '-1', 1, 'BINARY_SEARCH_NOT_FOUND'),
('tc_3_3_hid2', 'lvl_13_m3_busca_sentinela', '[[], 10]', '-1', 1, 'BINARY_SEARCH_EMPTY_LIST'),
-- Testes 3.4
('tc_3_4_pub1', 'lvl_14_m3_ranking', '[[["Beta", 10], ["Alfa", 25], ["Beta", 15], ["Gama", 20]], 2]', '["Alfa", "Beta"]', 0, 'RANKING_AGGREGATION_AND_TIEBREAK'),
('tc_3_4_hid1', 'lvl_14_m3_ranking', '[[["Z", 10], ["A", 10], ["M", 10]], 2]', '["A", "M"]', 1, 'RANKING_ALPHA_TIEBREAK'),
-- Testes 3.5 (Boss M3)
('tc_3_5_pub1', 'lvl_15_boss_modulo3', '[[{"id": "T2", "custo": 4, "prio": 2}, {"id": "T1", "custo": 3, "prio": 2}, {"id": "T3", "custo": 5, "prio": 1}, {"id": "INV", "custo": 0, "prio": 9}], 8]', '{"selecionadas": ["T1", "T2"], "custo_total": 7}', 0, 'SCHEDULER_FILTER_AND_SORT'),
('tc_3_5_hid1', 'lvl_15_boss_modulo3', '[[{"id": "B", "custo": 5, "prio": 3}, {"id": "A", "custo": 5, "prio": 3}, {"id": "C", "custo": 2, "prio": 1}], 7]', '{"selecionadas": ["A", "C"], "custo_total": 7}', 1, 'SCHEDULER_SKIP_TOO_HEAVY_TAKE_NEXT');

-- Progresso Inicial dos Módulos 2 e 3 (Hard Gatekeeping: desbloqueia ao vencer Boss anterior)
INSERT OR IGNORE INTO user_progress (id, profile_id, level_id, status) VALUES
('up_06', 'prof_local_01', 'lvl_06_m2_fatiamento', 'LOCKED'),
('up_07', 'prof_local_01', 'lvl_07_m2_dicionarios', 'LOCKED'),
('up_08', 'prof_local_01', 'lvl_08_m2_compreensao', 'LOCKED'),
('up_09', 'prof_local_01', 'lvl_09_m2_matrizes', 'LOCKED'),
('up_10', 'prof_local_01', 'lvl_10_boss_modulo2', 'LOCKED'),
('up_11', 'prof_local_01', 'lvl_11_m3_validacao', 'LOCKED'),
('up_12', 'prof_local_01', 'lvl_12_m3_janela_deslizante', 'LOCKED'),
('up_13', 'prof_local_01', 'lvl_13_m3_busca_sentinela', 'LOCKED'),
('up_14', 'prof_local_01', 'lvl_14_m3_ranking', 'LOCKED'),
('up_15', 'prof_local_01', 'lvl_15_boss_modulo3', 'LOCKED');

-- Insígnias para os Chefões dos Módulos 2 e 3
INSERT OR IGNORE INTO badges (id, code, title, description, icon_name, rule_condition_json) VALUES
('bdg_04', 'BOSS_SLAYER_M2', 'Mestre das Estruturas', 'Venceu o Desafio Chefão do Módulo 2 sem auxílio.', 'shield-check', '{"type": "BOSS_CLEARED", "module_id": "mod_02_estruturas"}'),
('bdg_05', 'BOSS_SLAYER_M3', 'Arquiteto de Algoritmos', 'Venceu o Desafio Chefão do Módulo 3 sem auxílio.', 'shield-check', '{"type": "BOSS_CLEARED", "module_id": "mod_03_algoritmos"}');
