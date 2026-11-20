// Component tests for the speech bubble (efficiency-familiar T39, R11.8, R11.9, R11.13, R12.5).
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import SpeechBubble from './SpeechBubble.svelte';

afterEach(() => {
  cleanup();
});

describe('SpeechBubble', () => {
  // Breaks if AI text is shown without the AI hint label, so learners cannot tell it from measured facts.
  it('labels AI text as an AI hint and says the code was sent to a provider', () => {
    render(SpeechBubble, {
      props: {
        speech: {
          kind: 'ready',
          label: 'AI hint',
          text: 'The nested scan looks quadratic.',
          sentToProvider: true
        }
      }
    });

    expect(screen.getByText('AI hint:')).toBeTruthy();
    expect(screen.getByText(/The nested scan looks quadratic\./)).toBeTruthy();
    expect(screen.getByText(/sent to a third-party AI provider/i)).toBeTruthy();
  });

  // Breaks if a template is dressed up as AI, or the caption that explains where it came from is missing.
  it('shows a template with the measured-results caption and no AI label', () => {
    render(SpeechBubble, {
      props: {
        speech: {
          kind: 'ready',
          label: 'Measured summary',
          text: 'Looks quadratic, medium confidence. The reference is linear.'
        }
      }
    });

    expect(screen.queryByText('AI hint:')).toBeNull();
    expect(screen.getByText('From your measured results')).toBeTruthy();
    expect(screen.queryByText(/third-party/i)).toBeNull();
  });

  // Breaks if text from the model can run as HTML in the learner's browser (P25).
  it('renders HTML in the body as plain text', () => {
    const { container } = render(SpeechBubble, {
      props: {
        speech: {
          kind: 'ready',
          label: 'AI hint',
          text: '<img src=x onerror=alert(1)><b>bold</b>',
          sentToProvider: true
        }
      }
    });

    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('b')).toBeNull();
    expect(container.textContent).toContain('<img src=x onerror=alert(1)>');
  });

  // Breaks if the failure line claims a result, or hides that the measured result above is still valid.
  it('shows the fixed unavailable line without a label', () => {
    render(SpeechBubble, {
      props: {
        speech: {
          kind: 'unavailable',
          text: 'The speech bubble is unavailable right now.'
        }
      }
    });

    expect(screen.getByText(/unavailable right now/)).toBeTruthy();
    expect(screen.queryByText('AI hint:')).toBeNull();
  });
});
