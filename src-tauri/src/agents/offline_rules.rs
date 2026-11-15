use crate::agents::{AstInspectionReport, SocraticHintResponse};

pub struct OfflineSocraticEngine;

impl OfflineSocraticEngine {
    pub fn diagnose(
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
        runtime_error: Option<&str>,
    ) -> SocraticHintResponse {
        if is_timeout_loop {
            return SocraticHintResponse {
                diagnostic_category: "INFINITE_LOOP".to_string(),
                friendly_explanation: "O seu código entrou em um laço contínuo e ultrapassou o limite de 2 segundos de execução.".to_string(),
                socratic_question: "Olhando para a condição do seu laço de repetição, a variável avaliada está sendo atualizada dentro do bloco para que a condição se torne falsa em algum momento?".to_string(),
                focus_line: 3,
                conceptual_reminder: "Todo laço while precisa atualizar sua variável de controle a cada iteração em direção à condição de parada.".to_string(),
                provider_used: "LOCAL_RUST_ENGINE".to_string(),
            };
        }

        if let Some(err) = runtime_error {
            if err.contains("RecursionError") {
                return SocraticHintResponse {
                    diagnostic_category: "RECURSION_ERROR".to_string(),
                    friendly_explanation: "Sua função chamou a si mesma muitas vezes seguidas e atingiu o limite de recursão.".to_string(),
                    socratic_question: "Qual é o caso base (condição de parada) que interrompe as chamadas recursivas e retorna um valor direto?".to_string(),
                    focus_line: 2,
                    conceptual_reminder: "Toda função recursiva precisa de um caso base alcançável para encerrar a pilha de chamadas.".to_string(),
                    provider_used: "LOCAL_RUST_ENGINE".to_string(),
                };
            }
            if err.contains("MemoryLimitError") {
                return SocraticHintResponse {
                    diagnostic_category: "MEMORY_LIMIT".to_string(),
                    friendly_explanation: "Seu algoritmo tentou alocar uma estrutura ou texto maior que o limite seguro do quiosque (64 MB).".to_string(),
                    socratic_question: "Você realmente precisa multiplicar ou construir uma sequência desse tamanho em memória para resolver o problema?".to_string(),
                    focus_line: 2,
                    conceptual_reminder: "Mantenha coleções e strings proporcionais ao tamanho dos dados de entrada.".to_string(),
                    provider_used: "LOCAL_RUST_ENGINE".to_string(),
                };
            }
            if err.contains("ZeroDivisionError") {
                return SocraticHintResponse {
                    diagnostic_category: "RUNTIME_ERROR".to_string(),
                    friendly_explanation: "O código tentou dividir um número por zero durante a execução.".to_string(),
                    socratic_question: "Em que situação o denominador da sua divisão pode assumir o valor 0 e como você pode tratar isso antes de dividir?".to_string(),
                    focus_line: 2,
                    conceptual_reminder: "Sempre verifique se o divisor é diferente de zero antes de realizar divisões.".to_string(),
                    provider_used: "LOCAL_RUST_ENGINE".to_string(),
                };
            }
            if err.contains("IndexError") {
                return SocraticHintResponse {
                    diagnostic_category: "RUNTIME_ERROR".to_string(),
                    friendly_explanation: "Seu código tentou acessar uma posição da lista que não existe para essa entrada.".to_string(),
                    socratic_question: "Lembre que listas começam no índice 0 e terminam em len(lista) - 1. O que acontece se a lista estiver vazia ou for menor?".to_string(),
                    focus_line: 2,
                    conceptual_reminder: "Ao percorrer vetores, prefira iterar diretamente sobre os elementos ou validar len(lista).".to_string(),
                    provider_used: "LOCAL_RUST_ENGINE".to_string(),
                };
            }
        }

        if let Some((err_msg, line)) = &ast_report.syntax_error_detail {
            return SocraticHintResponse {
                diagnostic_category: "SYNTAX_ERROR".to_string(),
                friendly_explanation: format!(
                    "O interpretador encontrou um detalhe de sintaxe na linha {} ({}).",
                    line, err_msg
                ),
                socratic_question: format!(
                    "Na linha {}, todos os dois-pontos (:), parênteses e recuos (indentação) estão alinhados corretamente?",
                    line
                ),
                focus_line: *line,
                conceptual_reminder: "Em Python, dois-pontos ao final de def, if, for e while indicam o início de um novo bloco indentado.".to_string(),
                provider_used: "LOCAL_RUST_ENGINE".to_string(),
            };
        }

        if !ast_report.triggered_forbidden_nodes.is_empty() {
            let forbidden = ast_report.triggered_forbidden_nodes.join(", ");
            return SocraticHintResponse {
                diagnostic_category: "AST_VIOLATION".to_string(),
                friendly_explanation: format!(
                    "Esta missão restringe o uso de certas instruções ({}) para garantir que você pratique o conceito central.",
                    forbidden
                ),
                socratic_question: "Como você pode reescrever sua lógica usando apenas as estruturas permitidas no enunciado da fase?".to_string(),
                focus_line: 2,
                conceptual_reminder: "Cada fase treina uma ferramenta lógica específica sem atalhos ou imports externos.".to_string(),
                provider_used: "LOCAL_RUST_ENGINE".to_string(),
            };
        }

        if ast_report.suspected_hardcode || !ast_report.missing_required_nodes.is_empty() {
            let missing = if ast_report.missing_required_nodes.is_empty() {
                "operação lógica sobre os parâmetros".to_string()
            } else {
                ast_report.missing_required_nodes.join(", ")
            };
            return SocraticHintResponse {
                diagnostic_category: "AST_VIOLATION".to_string(),
                friendly_explanation: format!(
                    "Sua solução ainda não utiliza a estrutura lógica exigida nesta fase ({}).",
                    missing
                ),
                socratic_question: "Como você pode transformar o valor recebido nos parâmetros usando a estrutura lógica desta missão em vez de retornar um valor fixo?".to_string(),
                focus_line: 2,
                conceptual_reminder: "Um algoritmo deve funcionar para qualquer entrada válida, e não apenas para um exemplo isolado.".to_string(),
                provider_used: "LOCAL_RUST_ENGINE".to_string(),
            };
        }

        let (question, reminder) = match failed_concept_tag.unwrap_or("GENERAL_LOGIC") {
            "ARITHMETIC_FORMULA" | "ZERO_BALANCE" | "NEGATIVE_NET_BALANCE" => (
                "Verifique a ordem dos termos no cálculo do saldo (geração menos consumo) e se o resultado está sendo multiplicado por 0.95 com round(..., 2).",
                "Parênteses definem a precedência entre subtração e multiplicação."
            ),
            "EXACT_BOUNDARY_INCLUSION_195" | "EXACT_BOUNDARY_INCLUSION_240" => (
                "O que acontece no seu código quando o valor de entrada é exatamente igual ao limite da faixa (195 ou 240)? Seu operador inclui a igualdade?",
                "Operadores < e <= produzem resultados diferentes exatamente nos valores de fronteira."
            ),
            "NOMINAL_RANGE" | "LOW_BOUNDARY" | "HIGH_BOUNDARY" => (
                "Compare as três faixas de tensão com atenção: valores abaixo de 195, acima de 240 e entre 195 e 240 retornam exatamente as palavras pedidas?",
                "Confira maiúsculas e ausência de acentos nas strings de retorno esperadas."
            ),
            "ALREADY_AT_OR_ABOVE_TARGET" => (
                "E se a carga inicial já for maior ou igual à meta máxima desde o início? Quantos ciclos deveriam ocorrer nesse caso?",
                "Um laço while avalia a condição antes da primeira execução do bloco."
            ),
            "WHILE_INCREMENT_COUNT" | "NON_EXACT_MULTIPLE_TARGET" => (
                "Observe quantas vezes o contador de ciclos é incrementado e se a carga atual sobe pelo incremento exato até atingir ou ultrapassar a meta.",
                "Acompanhe no Inspetor de Variáveis o valor de carga_atual e ciclos a cada volta do laço."
            ),
            "FILTER_POSITIVE_ONLY" | "ALL_INVALID_RETURNS_EMPTY_LIST" => (
                "O número 0 deve ser considerado estritamente positivo (> 0) ou ignorado? E o que sua função retorna quando nenhum item passa no filtro?",
                "Valores positivos são estritamente maiores que zero (> 0)."
            ),
            "SENTINEL_BREAK_AND_ACCUMULATION" | "IMMEDIATE_SENTINEL_AT_INDEX_ZERO" => (
                "Você está verificando o valor sentinela 9999 ANTES de somar a leitura ao total válido e de contabilizar os alertas?",
                "A condição do sentinela deve ser testada logo no início de cada iteração."
            ),
            "LIST_SLICING_SUM" | "SLICE_LARGER_THAN_LIST" | "EMPTY_LIST_GUARD" => (
                "Como seu código trata uma lista vazia antes de fatiar `[:k]` e `[-k:]`? Os dois somatórios estão separados corretamente?",
                "Fatiamentos negativos como `[-k:]` retornam os últimos k itens mesmo quando a lista tem menos de k elementos."
            ),
            "DICT_FREQUENCY_NORMALIZATION" | "IGNORE_EMPTY_NORMALIZED_STRINGS" => (
                "Você aplicou `.strip().upper()` antes de verificar se a string resultante está vazia e antes de incrementar a contagem no dicionário?",
                "Use `contagem[chave] = contagem.get(chave, 0) + 1` após normalizar e filtrar strings vazias."
            ),
            "LISTCOMP_FILTER_AND_UPPER" | "BOUNDARY_GRADE_70" => (
                "Na sua compreensão de listas, as duas condições (`ativo` verdadeiro e `nota >= 70`) estão sendo exigidas juntas com `and` e convertendo o nome para `.upper()`?",
                "Compreensões de listas aceitam filtros compostos na cláusula `if` ao final."
            ),
            "MATRIX_ROW_MAX_AND_EMPTY_ROW" | "MATRIX_NEGATIVE_NUMBERS" => (
                "O que acontece se uma linha da matriz estiver vazia ou contiver apenas números negativos? Você inicializou o máximo com `0` ou com o primeiro item da sublista?",
                "Nunca assuma `0` como máximo inicial para listas não vazias — use `max(linha)` ou o primeiro elemento."
            ),
            "SAFE_AVERAGE_INCLUSIVE_BOUNDS" | "NO_VALID_ELEMENTS_RETURNS_NONE" | "EXACT_INCLUSIVE_RANGE" => (
                "Sua comparação inclui exatamente `minimo` e `maximo` (`<=`) e retorna `None` antes de dividir quando nenhum item é válido?",
                "Dividir por zero ocorre quando o contador de válidos é 0."
            ),
            "SLIDING_WINDOW_MAX_SUM" | "WINDOW_EXCEEDS_LEN" | "NEGATIVE_WINDOW_SUMS" => (
                "Se a janela for maior que `len(serie)` ou `<= 0`, sua função retorna `0`? E para janelas válidas só com negativos, `melhor` inicia pela primeira janela?",
                "Inicialize a melhor soma com a soma da primeira janela real, e não com `0`."
            ),
            "BINARY_SEARCH_FOUND" | "BINARY_SEARCH_NOT_FOUND" | "BINARY_SEARCH_EMPTY_LIST" => (
                "O laço `while` mantém a condição `esq <= dir` e atualiza `esq = meio + 1` ou `dir = meio - 1` a cada passo para evitar loop infinito?",
                "Use divisão inteira `// 2` para calcular o índice do meio."
            ),
            "RANKING_AGGREGATION_AND_TIEBREAK" | "RANKING_ALPHA_TIEBREAK" | "SCHEDULER_FILTER_AND_SORT" | "SCHEDULER_SKIP_TOO_HEAVY_TAKE_NEXT" => (
                "Todos os critérios de desempate estão sendo aplicados na ordem correta (prioridade/pontos primeiro e critério alfabético no empate) antes do fatiamento final?",
                "Teste mentalmente dois itens com a mesma pontuação ou prioridade para verificar a ordenação."
            ),
            _ => (
                "Acompanhe no Inspetor de Variáveis à direita os valores passo a passo: em qual linha o resultado calculado se distancia do esperado pelo teste?",
                "Teste mentalmente seu código com valores de borda (zero, números negativos e limites exatos)."
            ),
        };

        SocraticHintResponse {
            diagnostic_category: "LOGIC_MISMATCH".to_string(),
            friendly_explanation: "A estrutura do seu código está válida, mas um dos casos de teste produziu um resultado diferente do esperado.".to_string(),
            socratic_question: question.to_string(),
            focus_line: 2,
            conceptual_reminder: reminder.to_string(),
            provider_used: "LOCAL_RUST_ENGINE".to_string(),
        }
    }
}
