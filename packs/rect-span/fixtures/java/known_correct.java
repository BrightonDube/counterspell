// Test-only known-correct Java fixture for rect-span.
public class Solution {
    public static long rect_span(long[] a, long[] b) {
        long ax1 = Math.min(a[0], a[2]), ax2 = Math.max(a[0], a[2]);
        long ay1 = Math.min(a[1], a[3]), ay2 = Math.max(a[1], a[3]);
        long bx1 = Math.min(b[0], b[2]), bx2 = Math.max(b[0], b[2]);
        long by1 = Math.min(b[1], b[3]), by2 = Math.max(b[1], b[3]);
        long w = Math.min(ax2, bx2) - Math.max(ax1, bx1);
        long h = Math.min(ay2, by2) - Math.max(ay1, by1);
        return Math.max(w, 0) * Math.max(h, 0);
    }
}
