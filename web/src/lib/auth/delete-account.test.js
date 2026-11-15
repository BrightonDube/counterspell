// Tests for the delete-account confirmation rule (auth T36, requirement R17.2). The rule is
// checked again on the server; this check only keeps the button disabled until the handle matches.
import { describe, expect, it } from 'vitest';
import { confirmationMatches } from './delete-account.js';

describe('delete account confirmation (auth T36)', () => {
  it('accepts only the exact handle', () => {
    expect(confirmationMatches('thandi', 'thandi')).toBe(true);
    expect(confirmationMatches('Thandi', 'thandi')).toBe(false);
    expect(confirmationMatches('thandi ', 'thandi')).toBe(false);
    expect(confirmationMatches('thand', 'thandi')).toBe(false);
  });

  it('never confirms when there is no handle to match', () => {
    expect(confirmationMatches('', '')).toBe(false);
    expect(confirmationMatches('x', undefined)).toBe(false);
    expect(confirmationMatches(undefined, 'thandi')).toBe(false);
  });
});
