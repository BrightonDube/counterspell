// Loads the language catalogue from `GET /api/languages`. Every language in the
// list is enabled; disabled stretch languages are never listed.
import { ContractError, asRecord } from '../contract';
import { getJson, type RequestOptions } from './client';
import { parseLanguageSummary, type LanguageSummary } from './types';

const MAX_LANGUAGES = 50;

/** The response is `{languages: [...]}`. Anything else is refused. */
function parseLanguageList(raw: unknown): LanguageSummary[] {
  const record = asRecord(raw, 'languages');
  const list = record.languages;
  if (!Array.isArray(list)) {
    throw new ContractError('languages', 'expected a languages array');
  }
  if (list.length > MAX_LANGUAGES) {
    throw new ContractError(
      'languages',
      `expected at most ${MAX_LANGUAGES} entries`
    );
  }
  return list.map((entry) => parseLanguageSummary(entry));
}

/** Fetches the languages the server lists. Each is enabled and supports both verbs. */
export function loadLanguages(
  options: RequestOptions = {}
): Promise<LanguageSummary[]> {
  return getJson('/api/languages', parseLanguageList, options);
}
