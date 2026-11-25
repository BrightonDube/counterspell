// Test-only known-correct Java fixture for even-offering.
public class Solution {
    private static boolean ok(long[] offers, long cap, long parts) {
        long need = 1, cur = 0;
        for (long v : offers) {
            if (cur + v > cap) { need += 1; cur = 0; }
            cur += v;
        }
        return need <= parts;
    }
    public static long smallest_largest(long[] offers, long parts) {
        long lo = 0, hi = 0;
        for (long v : offers) { if (v > lo) { lo = v; } hi += v; }
        while (lo < hi) {
            long mid = (lo + hi) / 2;
            if (ok(offers, mid, parts)) { hi = mid; } else { lo = mid + 1; }
        }
        return lo;
    }
}
