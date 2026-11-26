// Test-only known-correct Java fixture for meld-slate.
public class Solution {
    public static long[] meld(long[] a, long[] b) {
        long[] out = new long[a.length + b.length];
        int i = 0, j = 0, k = 0;
        while (i < a.length && j < b.length) {
            if (a[i] <= b[j]) {
                out[k++] = a[i++];
            } else {
                out[k++] = b[j++];
            }
        }
        while (i < a.length) {
            out[k++] = a[i++];
        }
        while (j < b.length) {
            out[k++] = b[j++];
        }
        return out;
    }
}
