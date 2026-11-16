import { afterEach, describe, expect, it, vi } from 'vitest';
import { afterPaint } from './after-paint';

describe('afterPaint (T15)', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.useRealTimers();
	});

	it('runs the callback in an idle period with a 1500 ms deadline when idle callbacks exist', () => {
		const requestIdleCallback = vi.fn((cb: () => void) => cb());
		vi.stubGlobal('requestIdleCallback', requestIdleCallback);
		const callback = vi.fn();

		afterPaint(callback);

		expect(requestIdleCallback).toHaveBeenCalledWith(expect.any(Function), { timeout: 1500 });
		expect(callback).toHaveBeenCalledTimes(1);
	});

	it('falls back to a 1500 ms timer when idle callbacks are missing', () => {
		vi.useFakeTimers();
		vi.stubGlobal('requestIdleCallback', undefined);
		const callback = vi.fn();

		afterPaint(callback);
		vi.advanceTimersByTime(1499);
		expect(callback).not.toHaveBeenCalled();
		vi.advanceTimersByTime(1);
		expect(callback).toHaveBeenCalledTimes(1);
	});
});
