// Loads the language catalogue from `GET /api/languages`. The server's `enabled` flags are authoritative.
import { ContractError } from '../contract';
import { getJson, type RequestOptions } from './client';
import { parseLanguageSummary, type LanguageSummary } from './types';

const MAX_LANGUAGES = 50;

/** The response is a JSON array of language summaries. Anything else is refused. */
function parseLanguageList(raw: unknown): LanguageSummary[] {
  if (!Array.isArray(raw)) {
    throw new ContractError('languages', 'expected an array');
  }
  if (raw.length > MAX_LANGUAGES) {
    throw new ContractError(
      'languages',
      `expected at most ${MAX_LANGUAGES} entries`
    );
  }
  return raw.map((entry) => parseLanguageSummary(entry));
}

/** Fetches the languages the server knows about, including disabled ones. */
export function loadLanguages(
  options: RequestOptions = {}
): Promise<LanguageSummary[]> {
  return getJson('/api/languages', parseLanguageList, options);
}
