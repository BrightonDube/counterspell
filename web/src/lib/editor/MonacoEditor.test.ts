// Component tests for the lazy Monaco editor (ide-workspace T07). Monaco is replaced by a fake that records
// what the component asks for, so each test names the editor behaviour it protects.
import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const harness = vi.hoisted(() => {
  type Listener = () => void;

  class FakeModel {
    value: string;
    language: string;
    disposed = false;
    constructor(value: string, language: string) {
      this.value = value;
      this.language = language;
    }
    getValue() {
      return this.value;
    }
    dispose() {
      this.disposed = true;
    }
  }

  class FakeEditor {
    options: Record<string, unknown>;
    model: FakeModel | null = null;
    listeners: Listener[] = [];
    disposed = false;
    constructor(options: Record<string, unknown>) {
      this.options = options;
    }
    setModel(model: FakeModel) {
      this.model = model;
    }
    getValue() {
      return this.model?.getValue() ?? '';
    }
    onDidChangeModelContent(listener: Listener) {
      this.listeners.push(listener);
      return { dispose() {} };
    }
    dispose() {
      this.disposed = true;
    }
  }

  const state = {
    models: [] as FakeModel[],
    editors: [] as FakeEditor[],
    failCreate: false
  };

  const editorApi = {
    create(_host: unknown, options: Record<string, unknown>) {
      if (state.failCreate) throw new Error('editor unavailable');
      const editor = new FakeEditor(options);
      state.editors.push(editor);
      return editor;
    },
    createModel(value: string, language: string) {
      const model = new FakeModel(value, language);
      state.models.push(model);
      return model;
    },
    defineTheme() {},
    setTheme() {}
  };

  return { state, editorApi, FakeModel, FakeEditor };
});

vi.mock('monaco-editor/editor/editor.api.js', () => ({
  editor: harness.editorApi,
  languages: {
    register() {},
    registerTokensProviderFactory() {},
    onLanguageEncountered() {}
  }
}));
vi.mock('monaco-editor/languages/definitions/python/register.js', () => ({}));
vi.mock(
  'monaco-editor/languages/definitions/javascript/register.js',
  () => ({})
);
vi.mock('monaco-editor/languages/definitions/java/register.js', () => ({}));
vi.mock('monaco-editor/languages/definitions/cpp/register.js', () => ({}));
vi.mock(
  'monaco-editor/editor/contrib/find/browser/findController.js',
  () => ({})
);
vi.mock('monaco-editor/editor/contrib/comment/browser/comment.js', () => ({}));
vi.mock(
  'monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js',
  () => ({})
);

import MonacoEditor from './MonacoEditor.svelte';

beforeEach(() => {
  harness.state.models.length = 0;
  harness.state.editors.length = 0;
  harness.state.failCreate = false;
});

afterEach(() => {
  cleanup();
});

describe('MonacoEditor', () => {
  // Breaks if the editor shows nothing while Monaco loads, or ignores the theme and the accessible label.
  it('shows a loading status, then creates a Crypt-themed editor with its label', async () => {
    render(MonacoEditor, {
      props: {
        value: 'print(1)',
        language: 'python',
        label: 'Python solution editor',
        onChange: () => {}
      }
    });
    expect(screen.getByRole('status').textContent).toContain('Loading editor');

    await waitFor(() => expect(harness.state.editors).toHaveLength(1));
    const options = harness.state.editors[0].options;
    expect(options.theme).toBe('crypt');
    expect(options.ariaLabel).toBe('Python solution editor');
    expect(harness.state.models[0].language).toBe('python');
    expect(harness.state.models[0].getValue()).toBe('print(1)');
  });

  // Breaks if the browser performance check cannot tell when the editor became usable (R15.5).
  it('marks monaco-ready once the editor exists, with no detail', async () => {
    const mark = vi.spyOn(performance, 'mark');
    try {
      render(MonacoEditor, {
        props: {
          value: 'print(1)',
          language: 'python',
          label: 'Python solution editor',
          onChange: () => {}
        }
      });
      expect(mark.mock.calls.map((c) => c[0])).not.toContain('monaco-ready');
      await waitFor(() => expect(harness.state.editors).toHaveLength(1));
      await waitFor(() =>
        expect(mark.mock.calls).toContainEqual(['monaco-ready'])
      );
    } finally {
      mark.mockRestore();
    }
  });

  // Breaks if switching language throws away the Python draft, so returning to Python loses work.
  it('keeps each language model when the learner switches language', async () => {
    const { rerender } = render(MonacoEditor, {
      props: {
        value: 'print(1)',
        language: 'python',
        label: 'Solution editor',
        onChange: () => {}
      }
    });
    await waitFor(() => expect(harness.state.editors).toHaveLength(1));

    await rerender({
      value: 'console.log(1)',
      language: 'javascript',
      label: 'Solution editor',
      onChange: () => {}
    });
    await waitFor(() => expect(harness.state.models).toHaveLength(2));
    expect(harness.state.editors[0].model?.language).toBe('javascript');
    expect(harness.state.editors[0].getValue()).toBe('console.log(1)');

    await rerender({
      value: 'stale draft',
      language: 'python',
      label: 'Solution editor',
      onChange: () => {}
    });
    expect(harness.state.models).toHaveLength(2);
    expect(harness.state.editors[0].getValue()).toBe('print(1)');
  });

  // Breaks if the parent never hears about typing, so the Run and Submit controls would send stale source.
  it('reports each source change to its parent', async () => {
    const onChange = vi.fn();
    render(MonacoEditor, {
      props: {
        value: 'x = 1',
        language: 'python',
        label: 'Solution editor',
        onChange
      }
    });
    await waitFor(() => expect(harness.state.editors).toHaveLength(1));

    const editor = harness.state.editors[0];
    const model = editor.model;
    if (!model) throw new Error('the editor should have a model');
    model.value = 'x = 2';
    editor.listeners.forEach((listener) => listener());
    expect(onChange).toHaveBeenLastCalledWith('x = 2');
  });

  // Breaks if the editor and its models leak after the learner leaves the workspace.
  it('disposes the editor and every model on unmount', async () => {
    const { rerender } = render(MonacoEditor, {
      props: {
        value: 'print(1)',
        language: 'python',
        label: 'Solution editor',
        onChange: () => {}
      }
    });
    await waitFor(() => expect(harness.state.editors).toHaveLength(1));
    await rerender({
      value: 'console.log(1)',
      language: 'javascript',
      label: 'Solution editor',
      onChange: () => {}
    });
    await waitFor(() => expect(harness.state.models).toHaveLength(2));

    cleanup();
    expect(harness.state.editors[0].disposed).toBe(true);
    expect(harness.state.models.every((model) => model.disposed)).toBe(true);
  });

  // Breaks if a failed editor start is silent, which leaves the learner with a blank panel.
  it('shows a failure message with a retry button when Monaco cannot start', async () => {
    harness.state.failCreate = true;
    render(MonacoEditor, {
      props: {
        value: 'print(1)',
        language: 'python',
        label: 'Solution editor',
        onChange: () => {}
      }
    });
    await waitFor(() =>
      expect(screen.getByRole('alert').textContent).toContain('could not load')
    );
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy();
  });
});
